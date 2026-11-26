//! The sky pass: `Main.SetBackColor`'s sky colours, the deterministic cloud/star
//! assets and `DrawSunAndMoon`/`DrawSurfaceBG` (`draw_sky`). The textured
//! underground backdrop it tails into lives in [`super::bg`].
use super::BgTex;
use super::bg::{
    FrontLayer, bg_scale_adjust, draw_background, draw_bg_layer, draw_front_trees,
    forest_173_frame_id, forest_layers, front_layers, layer_y_offset, mushroom_tint,
    push_screen_rot,
};
use crate::app::next_rand;
use crate::assets;
use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::{TILE_PX, World};
use tr_render::{Atlas, SpriteRenderer};

/// The debug font is ASCII-only; transliterate Cyrillic names so they stay readable.
pub(crate) fn display_name(name: &str) -> String {
    const CYRILLIC: &str = "АБВГДЕЁЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯабвгдеёжзийклмнопрстуфхцчшщъыьэюя";
    const LATIN: &[&str] = &[
        "A", "B", "V", "G", "D", "E", "Yo", "Zh", "Z", "I", "Y", "K", "L", "M", "N", "O", "P", "R",
        "S", "T", "U", "F", "Kh", "Ts", "Ch", "Sh", "Shch", "", "Y", "", "E", "Yu", "Ya", "a", "b",
        "v", "g", "d", "e", "yo", "zh", "z", "i", "y", "k", "l", "m", "n", "o", "p", "r", "s", "t",
        "u", "f", "kh", "ts", "ch", "sh", "shch", "", "y", "", "e", "yu", "ya",
    ];
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if let Some(idx) = CYRILLIC.chars().position(|c| c == ch) {
            out.push_str(LATIN[idx]);
        } else if ch.is_ascii() {
            out.push(ch);
        } else {
            out.push('?');
        }
    }
    out
}

/// `Main.ApplyColorOfTheSkiesToTiles` / the sky ramps moved to
/// [`tr_core::sky_color`] (W4-g70 DOC-SWEEP lane 8): the full `SetBackColor`
/// chain now runs in XNA byte space there; this module keeps the draw-side
/// horizon blend and the celestial/parallax passes.
/// `Main.SetBackColor`: the colour the sky and surface background are tinted with
/// (`Main.ColorOfTheSkies`), as a function of world time.
///
/// Day runs `0..54000` (0 = sunrise, 27000 = noon), night runs `0..32400`
/// (16200 = midnight). Between 13500 and 37800 the colour is pure white, which is why
/// vanilla daytime tiles are lit at 100 %.
pub(crate) fn color_of_the_skies(
    time: f64,
    day: bool,
    moon_phase: u8,
    blood_moon: bool,
) -> [f32; 3] {
    let byte = |v: f64| (v.clamp(0.0, 255.0) / 255.0) as f32;
    if day {
        if time < 13500.0 {
            let t = time / 13500.0;
            [
                byte(t * 230.0 + 25.0),
                byte(t * 220.0 + 35.0),
                byte(t * 220.0 + 35.0),
            ]
        } else if time > 45900.0 {
            let t = (1.0 - (time / 54000.0 - 0.85) * 6.666_666_666_666_667).clamp(0.0, 1.0);
            [
                byte(t * 200.0 + 35.0),
                byte(t * 85.0 + 35.0),
                byte(t * 135.0 + 35.0),
            ]
        } else if time > 37800.0 {
            let t = (1.0 - (time / 54000.0 - 0.7) * 6.666_666_666_666_667).clamp(0.0, 1.0);
            [
                byte(t * 20.0 + 235.0),
                byte(t * 135.0 + 120.0),
                byte(t * 85.0 + 170.0),
            ]
        } else {
            [1.0, 1.0, 1.0]
        }
    } else {
        // `Main.cs:63493-63540`: the two night ramps, then the moon-phase light
        // floor — `Main.cs:63792-63813` grades `minimalLight` 11 (new moon) up
        // to 19 (full moon), applied per channel at `:63818-63829`. The
        // blood-moon arm (`:63495-63519`) replaces BOTH ramps with the red
        // sky (`tr_core::sky::blood_moon_sky`) and its floor is 25
        // (`:63830-63844`), applied AFTER the moon-phase floor — the
        // `BloodMoonMonolith` half of the C# gate (`Main.cs:61979`) is
        // unported, the caller passes `World.blood_moon` (W4-g12 doc-sweep).
        let (r, g, b) = if blood_moon {
            let c = tr_core::sky::blood_moon_sky(time);
            (c[0] as f64, c[1] as f64, c[2] as f64)
        } else if time < 16200.0 {
            let t = 1.0 - time / 16200.0;
            (t * 30.0 + 5.0, t * 30.0 + 5.0, t * 30.0 + 5.0)
        } else {
            let t = (time / 32400.0 - 0.5) * 2.0;
            (t * 20.0 + 5.0, t * 30.0 + 5.0, t * 30.0 + 5.0)
        };
        // `Enums/MoonPhase.cs`: Full=0, Empty=4, the right-half mirror behind.
        const FLOORS: [f64; 8] = [19.0, 17.0, 15.0, 13.0, 11.0, 13.0, 15.0, 17.0];
        let floor = (FLOORS[(moon_phase & 7) as usize] / 255.0) as f32;
        // `:63830-63844`: the blood-moon floor is a BYTE compare (< 25)
        // on the already-cast channels, after the moon-phase floor.
        let bm_floor = if blood_moon { 25.0 / 255.0 } else { 0.0 };
        [
            byte(r).max(floor).max(bm_floor),
            byte(g).max(floor).max(bm_floor),
            byte(b).max(floor).max(bm_floor),
        ]
    }
}

/// `SetBackColor`'s rain dim (`Main.cs:63546-63552`): while
/// `cloudAlpha > 0 && !remixWorld`, every sky channel is multiplied by
/// `1 - cloudAlpha * 0.9 * atmo` — byte-truncated (`(byte)((int)ch * num3)`).
/// W4-g69 doc-sweep: the RAW cloudAlpha (the port's clamped `visual_alpha`
/// was wrong for over-1 pins), the atmo factor and the remix exclusion were
/// missing, and the retired 0.1 floor is not in the C#. The per-cloud
/// darkening is NOT here: `Cloud.cloudColor` carries no rain tint
/// (`Cloud.cs:227-235`); the dark storm TYPES 18-21 (`Cloud.cs:116-127`)
/// plus this GLOBAL sky dim are the whole rain-cloud darkening.
pub(crate) fn rain_dim_sky(sky: [f32; 3], cloud_alpha: f32, atmo: f32, remix: bool) -> [f32; 3] {
    if cloud_alpha <= 0.0 || remix {
        return sky;
    }
    let num3 = 1.0 - cloud_alpha * 0.9 * atmo;
    // `(byte)((float)(int)ch * num3)` — the XNA byte cast. In the C#'s
    // unchecked context a negative product wraps through the low byte
    // (`as i32 as u8`, the `cloud_color` convention); only a >1.11 cloudAlpha
    // pin reaches it (vanilla maxRaining ≤ 1 keeps num3 ≥ 0.1). The channel
    // bytes come from the same `(byte)(float * 255)` truncation
    // `draw_cloud_color` uses for `Main.ColorOfTheSkies`.
    let ch = |v: f32| ((v * 255.0) as i32 as u8 as f32 * num3) as i32 as u8 as f32 / 255.0;
    [ch(sky[0]), ch(sky[1]), ch(sky[2])]
}

/// `Main.ApplyColorOfTheSkiesToTiles`: surface sunlight is the sky colour pulled
/// 70 % towards its own channel.
pub(crate) fn tile_color(sky: [f32; 3]) -> [f32; 3] {
    let sum = sky[0] + sky[1] + sky[2];
    [
        (sum + 7.0 * sky[0]) / 10.0,
        (sum + 7.0 * sky[1]) / 10.0,
        (sum + 7.0 * sky[2]) / 10.0,
    ]
}

/// `SunGradients` (`GameContent/Drawing/SunGradients.cs:37-197`): the 15-entry
/// gradient tables are hardcoded colours; only the middle entry
/// (`gradient[gradient.Length / 2]`, `NextHorizonRenderer.cs:185`) feeds the
/// light tint, so the port stores just that per table, in the vanilla list
/// order (`:183-197`): Sunrises [Blue, Violet, Yellow, Aluminum], Sunsets
/// [Blue, Dark, Pink, Red].
mod sun_gradients {
    /// `(r, g, b)` 0..255.
    pub type Gradient = [f32; 3];

    pub const SUNRISES: [Gradient; 4] = [
        [68.0, 144.0, 149.0],  // _Sunrise_Blue[7]
        [106.0, 92.0, 142.0],  // _Sunrise_Violet[7]
        [69.0, 92.0, 102.0],   // _Sunrise_Yellow[7]
        [138.0, 95.0, 155.0],  // _Sunrise_Aluminum[7]
    ];
    pub const SUNSETS: [Gradient; 4] = [
        [192.0, 106.0, 115.0], // _Sunset_Blue[7]
        [47.0, 23.0, 45.0],    // _Sunset_Dark[7]
        [173.0, 93.0, 145.0],  // _Sunset_Pink[7]
        [64.0, 32.0, 45.0],    // _Sunset_Red[7]
    ];
}

/// `Main.HorizonPhase` (`Main.cs:2792-2805`): the moon phase, decremented
/// before mid-day so the morning half belongs to the previous phase's horizon.
pub(crate) fn horizon_phase(moon_phase: u8, day: bool, time: f64) -> usize {
    let mut phase = moon_phase as i32;
    if day && time < 27000.0 {
        phase -= 1;
    }
    if phase < 0 {
        phase += 8;
    }
    phase as usize
}

/// `Main.UpdateAtmosphereTransparencyToSkyColor` (`Main.cs:63185-63210`): the
/// depth factor the horizon visibilities ride. `y` is the camera centre in
/// world pixels (`Main.cs:61983`).
pub(crate) fn atmosphere_factor(y: f32, world_surface: f64, max_tiles_x: i32) -> f32 {
    // `Main.cs:63187-63188`: the numerator computes in f32, the division by
    // `worldSurface / 5.0` in f64, the result casts back to f32 before the
    // clamps (the `tr_core::sky::atmosphere_transparency` shape).
    let n = max_tiles_x as f32 / 4200.0;
    let numer = y / 16.0 - (65.0 + 10.0 * n * n);
    let atmo = (numer as f64 / (world_surface / 5.0)) as f32;
    atmo.clamp(0.0, 1.0)
}

/// `NextHorizonRenderer.GetVisibilities` (`NextHorizonRenderer.cs:196-234`):
/// the sunrise/sunset light-tint strengths, verbatim in f32 (the C# floats).
/// The mushroom factor (`:203`) and the eclipse kill (`:219-223`) are live
/// inputs now (W4-g70 lane 8). The `gameMenu && drunkWorldGen` pair
/// (`:230-233`) only fires while generating — the port's menus draw their own
/// backdrop, so it stays out.
pub(crate) fn horizon_visibilities(
    time: f64,
    day: bool,
    atmo: f32,
    cloud_alpha: f32,
    mushroom: f32,
    eclipse: bool,
) -> (f32, f32) {
    let mut sunrise = 1.0f32;
    let mut sunset = 1.0f32;
    // `:201-203`: `num = 1f * atmo; num2 = 1f - cloudAlpha;
    // num = num * (num2 * num2) * (1f - mushroom)`.
    let num2 = 1.0 - cloud_alpha;
    let num = atmo * (num2 * num2) * (1.0 - mushroom);
    sunrise *= num;
    sunset *= num;
    let remap32 = |v: f32, lo: f32, hi: f32, a: f32, z: f32| {
        a + (z - a) * ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
    };
    let t = time as f32;
    if day {
        // Dawn ramp in, morning ramp out; the sunset mirrors it around the
        // afternoon (all windows in game ticks, `:215-218`).
        sunrise *= remap32(t, 0.0, 2700.0, 0.0, 1.0) * remap32(t, 3600.0, 10800.0, 1.0, 0.0);
        let s = remap32(t, 43200.0, 50400.0, 0.0, 1.0);
        let e = remap32(t, 50400.0, 54000.0, 1.0, 0.0);
        sunset *= s * e * e;
        if eclipse {
            sunset = 0.0;
            sunrise = 0.0;
        }
    } else {
        sunrise = 0.0;
        sunset = 0.0;
    }
    (sunset, sunrise)
}

/// `NextHorizonRenderer.BlendColor` (`NextHorizonRenderer.cs:183-194`) +
/// `Main.instance.HorizonRenderer.ModifyHorizonLight` (`Main.cs:63921`): the
/// mid gradient entries blend into the sky colour at the visibility strengths
/// — the "HorizonClouds tint". Byte space: `Color.Lerp` truncates each channel
/// (`(byte)(a + (b - a) * amount)`), and the max-lift target is per channel.
/// The `ShouldDrawSurfaceBackground` gate (`NextHorizonRenderer.cs:156`) is
/// the caller's `background_enabled && !remix` (the map-fullscreen case has
/// no sky pass in the port).
pub(crate) fn modify_horizon_light(
    color: &mut [u8; 3],
    moon_phase: u8,
    day: bool,
    time: f64,
    atmo: f32,
    cloud_alpha: f32,
    mushroom: f32,
    eclipse: bool,
) {
    let (sunset, sunrise) = horizon_visibilities(time, day, atmo, cloud_alpha, mushroom, eclipse);
    let idx = horizon_phase(moon_phase, day, time) % 4;
    let blend = |c: &mut [u8; 3], grad: [f32; 3], amount: f32| {
        if amount <= 0.0 {
            return;
        }
        for (ch, g) in c.iter_mut().zip(grad) {
            let target = (*ch as f32).max(g);
            *ch = (*ch as f32 + (target - *ch as f32) * amount) as u8;
        }
    };
    blend(color, sun_gradients::SUNSETS[idx], sunset);
    blend(color, sun_gradients::SUNRISES[idx], sunrise);
}

/// `[lo, hi)` draw from the splitmix stream — the shape of `Random.Next(lo, hi)`.
fn next_range(state: &mut u64, lo: i32, hi: i32) -> i32 {
    lo + (next_rand(state) % (hi - lo) as u64) as i32
}

// The live cloud field (`crate::cloud::CloudField`, W4-g69) replaced the
// retired `make_clouds`/`CloudSprite` stand-in: `Cloud.addCloud`'s spawn RNG,
// the wind drift, the rain-cloud darkening and the camera parallax now run as
// the C# does; the draw below consumes the 200-slot pool directly.

/// Default-derivable: every sheet is an `Option` guard (`None` draws nothing),
/// which is also what the headless flow tests construct.
#[derive(Default)]
pub(crate) struct SkyAssets {
    /// Snow biome: mountains `Main.snowMntBG` (default `Background_35/36`),
    /// pine line `Main.snowBG` (default `Background_37..39`), all resolved
    /// from the world's `setBG(3)` style (W4-g71). The old fixed forest
    /// `far`/`near`/`forest` fields retired onto the four-set
    /// `forest_mnt`/`forest_sets` tables below (W4-g70).
    pub(crate) snow_far: Option<BgTex>,
    pub(crate) snow_near: Option<BgTex>,
    pub(crate) snow_trees: [Option<BgTex>; 3],
    /// Ocean horizon (`Main.oceanBG`, default `Background_28`), resolved from
    /// the world's `setBG(7)` style.
    pub(crate) ocean: Option<BgTex>,
    /// Desert dunes: the Pure variant's far/middle rows (`Background_21/20`
    /// at style 0) plus the mountain layers behind them (`Background_23`
    /// far, `Background_22` near). The three evil/hallow variants live in
    /// `desert_variants`.
    pub(crate) desert_far: Option<BgTex>,
    pub(crate) desert_near: Option<BgTex>,
    /// The Pure variant's third row (`Background_250`, desert style 4 only).
    pub(crate) desert_third: Option<BgTex>,
    pub(crate) desert_far_mnt: Option<BgTex>,
    pub(crate) desert_near_mnt: Option<BgTex>,
    /// `Main.desertBackgroundSet.{Corrupt,Crimson,Hallow}` — the three rows
    /// `DrawSurfaceBG_GoodEvilDesert` swaps in for front styles 5/14/13
    /// (`Main.cs:60868-60884`). Index order matches `World::desert_bg_set`
    /// minus Pure: [Corrupt, Crimson, Hallow].
    pub(crate) desert_variants: [[Option<BgTex>; 3]; 3],
    /// The GoodEvilDesert fallback dunes (`Background_26/27`,
    /// `Main.cs:60885-60925`) — drawn when the style's variant set is empty.
    pub(crate) desert_fallback: [Option<BgTex>; 2],
    /// Storm cloud banks (`Background_112/113`), faded in by `cloudBGAlpha`.
    pub(crate) storm_far: Option<BgTex>,
    pub(crate) storm_near: Option<BgTex>,
    // Per-biome foreground layers, indexed by style variant then layer (0-2).
    // These are loaded on demand from the atlas by texture ID; each biome's
    // `*_bg` field on `World` holds the 3 active C# texture IDs.
    /// `DrawSurfaceBG_Corrupt` textures (`Background_12/13/14` style 0 default).
    pub(crate) corrupt_layers: [Option<BgTex>; 3],
    /// `DrawSurfaceBG_Jungle` (style 0: `Background_15/16/17`).
    pub(crate) jungle_layers: [Option<BgTex>; 3],
    /// `DrawSurfaceBG_Hallow` (style 0: `Background_29/30/31`).
    pub(crate) hallow_layers: [Option<BgTex>; 3],
    /// `DrawSurfaceBG_Crimson` (style 0: `Background_43/44/45`).
    pub(crate) crimson_layers: [Option<BgTex>; 3],
    /// `DrawSurfaceBG_Mushroom` (style 0: `Background_46/47/48`).
    pub(crate) mushroom_layers: [Option<BgTex>; 3],
    /// W4-g70: the four forest sets' tree lines (`Main.treeBGSet1..4`,
    /// resolved from `World.forest_bg_styles` through `SetForestBGSet`).
    /// Set 1 style 0 is the classic `Background_9/10/11` row.
    pub(crate) forest_sets: [[Option<BgTex>; 3]; 4],
    /// The four forest sets' hill pairs (`Main.treeMntBGSet1..4` at `setBG`
    /// time; the live animated ids ride `App.forest_hills` and resolve
    /// through `forest_mnt_anim` at draw).
    pub(crate) forest_mnt: [[Option<BgTex>; 2]; 4],
    /// The animated hill families' sheets, indexed by id: 94/114/115/116
    /// (near) + 93/168/169/170 (far) and 180..183 (near). `None` when the
    /// world's styles never reach them (the atlas packs them only then).
    pub(crate) forest_mnt_anim: std::collections::HashMap<i32, BgTex>,
    /// The 173 tree line's four draw-time frames (`Background_251..254`,
    /// `Main.cs:61373`), packed only when a set's row 0 is 173.
    pub(crate) forest_173_frames: [Option<BgTex>; 4],
    /// `Background_1`: the one-row surface transition into the underground backdrop.
    pub(crate) underground_transition: Option<BgTex>,
    /// `Background_2`: the tiled dirt backdrop (`_drawBackground_backTexture[1]`).
    pub(crate) underground: Option<BgTex>,
    /// `Background_3`: the rock-layer bands (`_drawBackground_backTexture[3]`, style 0).
    pub(crate) rock: Option<BgTex>,
    /// `Background_4`: the one-row band that closes the dirt sheet at the rock
    /// transition (`_drawBackground_backTexture[2]`, style 0).
    pub(crate) rock_band: Option<BgTex>,
    /// `Background_6`: the rock→magma transition band
    /// (`_drawBackground_backTexture[4]`, style 0).
    pub(crate) magma_transition: Option<BgTex>,
    /// `Background_125..127` (`128px wide`, three 96 px frames): the animated magma
    /// field, picked by `Main.hellBackStyle` (`Main.cs:54000-54007`).
    pub(crate) magma: [Option<BgTex>; 3],
    /// `Background_185..187`: the hell floor row, picked by `Main.hellBackStyle`
    /// (`Main.cs:54008`).
    pub(crate) hell_bottom: [Option<BgTex>; 3],
    /// `Images/Black_Tile`: the solid black sheet of
    /// `DrawBackground_DrawUnderworldBlackBox`.
    pub(crate) black_tile: Option<BgTex>,
    pub(crate) sun: Option<BgTex>,
    /// `Images/Sun3`: the eclipse sun (`Main.cs:62930`).
    pub(crate) sun_eclipse: Option<BgTex>,
    /// `Images/Sun2`: the Sunglasses-head sun (`Main.cs:62935`, head == 12).
    pub(crate) sun_sunglasses: Option<BgTex>,
    /// `Images/Extra_280`: the sunglasses overlay drawn at alpha 100 over the
    /// Sun2 body (`Main.cs:62950-62952`).
    pub(crate) sunglasses_overlay: Option<BgTex>,
    /// `Images/Moon_0..8` (50x400: eight 50 px phase rows); `Main.moonType` picks
    /// the sheet, `Main.moonPhase` the row.
    pub(crate) moon: [Option<BgTex>; 9],
    /// `Images/Moon_Smiley`: the drunk seed's whole-frame moon
    /// (`Main.cs:62964-62967`, rotation `num9/2 + π`).
    pub(crate) moon_smiley: Option<BgTex>,
    /// `Images/Moon_Pumpkin` / `Moon_Snow`: the event moons' phase-row sheets
    /// (`Main.cs:62968-62975`), same 8-row layout as `Moon_0..8`.
    pub(crate) moon_pumpkin: Option<BgTex>,
    pub(crate) moon_snow: Option<BgTex>,
    /// `Images/Misc/Sunflare/flare1/flare2`: `NextHorizonRenderer.DrawSun`'s
    /// five flare quads (`NextHorizonRenderer.cs:168-181`).
    pub(crate) sunflare1: Option<BgTex>,
    pub(crate) sunflare2: Option<BgTex>,
    pub(crate) stars: Vec<Option<BgTex>>,
    /// W4-f39: `Images/Misc/Sky_Slime_1..4` — the SlimeSky's falling-slime
    /// sheets (`SlimeSky.cs:79-83`), each a 4-row animation strip.
    pub(crate) slime_sky: [Option<BgTex>; 4],
}

/// The live star pool's element is [`crate::star_field::Star`] (W4-g70
/// lane 8); the draw below reads its per-tick twinkle/rotation directly.

pub(crate) fn sky_tex(
    atlas: &Atlas,
    name: &str,
    sub: Option<(u32, u32, u32, u32)>,
) -> Option<BgTex> {
    let rect = atlas.sheet(name)?;
    let (sx, sy, w, h) = sub.unwrap_or((0, 0, rect.width, rect.height));
    if sx + w > rect.width || sy + h > rect.height {
        return None;
    }
    Some(BgTex {
        uv: [
            (rect.x + sx) as f32 / atlas.width as f32,
            (rect.y + sy) as f32 / atlas.height as f32,
            (rect.x + sx + w) as f32 / atlas.width as f32,
            (rect.y + sy + h) as f32 / atlas.height as f32,
        ],
        w: w as f32,
        h: h as f32,
    })
}

pub(crate) fn make_sky_assets(atlas: &Atlas, world: &World) -> SkyAssets {
    // The eight surface variants draw whatever `WorldGen.setBG` rolled for
    // this world (`Main.corruptBG` etc.); `-1` means the C# layer is skipped.
    let layers = |ids: [i32; 3]| {
        ids.map(|i| {
            if i >= 0 {
                sky_tex(atlas, &format!("Background_{i}"), None)
            } else {
                None
            }
        })
    };
    let one = |id: i32| {
        if id >= 0 {
            sky_tex(atlas, &format!("Background_{id}"), None)
        } else {
            None
        }
    };
    let snow_mnts = world.snow_mountains();
    let desert = world.desert_bg_set();
    SkyAssets {
        // W4-g71: the snow/ocean/desert sheets resolve from the world's
        // live `setBG(3/6/7)` styles (the fixed 35/36/37..39/28/21/20
        // stand-ins retired); the forest sets ride the g70 four-set tables
        // below (the old fixed far/near/forest fields dropped).
        snow_far: one(snow_mnts[0]),
        snow_near: one(snow_mnts[1]),
        snow_trees: layers(world.snow_bg()),
        ocean: one(world.ocean_bg()),
        desert_far: one(desert[0][0]),
        desert_near: one(desert[0][1]),
        desert_third: one(desert[0][2]),
        desert_far_mnt: sky_tex(atlas, "Background_23", None),
        desert_near_mnt: sky_tex(atlas, "Background_22", None),
        // [Corrupt, Crimson, Hallow] — `World::desert_bg_set`'s rows 1..3.
        desert_variants: [layers(desert[1]), layers(desert[2]), layers(desert[3])],
        desert_fallback: [26, 27].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        storm_far: sky_tex(atlas, "Background_112", None),
        storm_near: sky_tex(atlas, "Background_113", None),
        corrupt_layers: layers(world.corrupt_bg()),
        jungle_layers: layers(world.jungle_bg()),
        hallow_layers: layers(world.hallow_bg()),
        crimson_layers: layers(world.crimson_bg()),
        mushroom_layers: layers(world.mushroom_bg()),
        forest_sets: std::array::from_fn(|n| layers(world.forest_bg(n).1)),
        forest_mnt: std::array::from_fn(|n| {
            world
                .forest_bg(n)
                .0
                .map(|i| sky_tex(atlas, &format!("Background_{i}"), None))
        }),
        forest_mnt_anim: [93, 94, 114, 115, 116, 168, 169, 170, 180, 181, 182, 183]
            .into_iter()
            .filter_map(|i| sky_tex(atlas, &format!("Background_{i}"), None).map(|t| (i, t)))
            .collect(),
        forest_173_frames: [251, 252, 253, 254]
            .map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        underground_transition: sky_tex(atlas, "Background_1", None),
        underground: sky_tex(atlas, "Background_2", None),
        rock: sky_tex(atlas, "Background_3", None),
        rock_band: sky_tex(atlas, "Background_4", None),
        magma_transition: sky_tex(atlas, "Background_6", None),
        magma: [125, 126, 127].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        hell_bottom: [185, 186, 187].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        black_tile: sky_tex(atlas, "Black_Tile", None),
        sun: sky_tex(atlas, "Sun", None),
        sun_eclipse: sky_tex(atlas, "Sun3", None),
        sun_sunglasses: sky_tex(atlas, "Sun2", None),
        sunglasses_overlay: sky_tex(atlas, "Extra_280", None),
        moon: std::array::from_fn(|i| sky_tex(atlas, &format!("Moon_{i}"), None)),
        moon_smiley: sky_tex(atlas, "Moon_Smiley", None),
        moon_pumpkin: sky_tex(atlas, "Moon_Pumpkin", None),
        moon_snow: sky_tex(atlas, "Moon_Snow", None),
        sunflare1: sky_tex(atlas, "Misc/Sunflare/flare1", None),
        sunflare2: sky_tex(atlas, "Misc/Sunflare/flare2", None),
        stars: (0..4)
            .map(|i| sky_tex(atlas, &format!("Star_{i}"), None))
            .collect(),
        slime_sky: std::array::from_fn(|i| sky_tex(atlas, &format!("Misc/Sky_Slime_{}", i + 1), None)),
    }
}

/// The static star field (`make_stars`/`StarSprite`) is retired (W4-g70
/// lane 8): the live pool lives in [`crate::star_field`] — `Star.Update`
/// steps the twinkle/rotation per tick, so the draw reads the live values
/// instead of re-deriving them from `anim_time`.

/// `NextHorizonRenderer.GetCelestialEffectPower`
/// (`NextHorizonRenderer.cs:311-321`): the flare strength ramps in over the
/// first 3600 ticks and out over the last 1800 of each half-day.
pub(crate) fn celestial_effect_power(time: f64, day: bool) -> f32 {
    let remap = |v: f32, lo: f32, hi: f32, a: f32, z: f32| {
        a + (z - a) * ((v - lo) / (hi - lo)).clamp(0.0, 1.0)
    };
    let t = time as f32;
    if day {
        remap(t, 0.0, 3600.0, 0.0, 1.0) * remap(t, 52200.0, 54000.0, 1.0, 0.0)
    } else {
        remap(t, 0.0, 3600.0, 0.0, 1.0) * remap(t, 30600.0, 32400.0, 1.0, 0.0)
    }
}

/// `DrawClouds_Distant`'s cY (`Main.cs:59682-59703`): the UNSCALED cloud Y
/// over `num = backgroundTopMagicNumber * 750 + 830 + scAdj + pushBGTopHack`
/// (the distant call passes hack 0, `Main.cs:59334`).
pub(crate) fn cloud_distant_cy(y: f32, magic: f32, sc_adj: f32) -> f32 {
    y + (magic * 750.0 + 830.0 + sc_adj)
}

/// `DrawClouds_Closer`'s cY (`Main.cs:59663-59680`):
/// `Y * (screenHeight / 600) + num + 200`, `num = bgTopY - 50`.
pub(crate) fn cloud_closer_cy(y: f32, screen_h: f32, bg_top_y: f32) -> f32 {
    y * (screen_h / 600.0) + (bg_top_y - 50.0) + 200.0
}

/// `DrawClouds_Closest`'s cY (`Main.cs:59644-59661`):
/// `Y * (screenHeight / 600) - 100 + num`, `num = bgTopY * 1.01 - 150`.
pub(crate) fn cloud_closest_cy(y: f32, screen_h: f32, bg_top_y: f32) -> f32 {
    y * (screen_h / 600.0) - 100.0 + (bg_top_y * 1.01 - 150.0)
}

/// The `DrawCloud` colour chain (`NextHorizonRenderer.cs:241-252`), verbatim
/// byte math: `Cloud.cloudColor(ColorOfTheSkies)` (`Cloud.cs:227-235`, every
/// channel including alpha truncated through the byte cast), the distant
/// pass's red/green pull (`OriginalColorsForCloud`, `:254-263`), the
/// `atmo < 1` dim (`:247-250`, XNA's float `Color * float` truncates each
/// channel through the byte cast) and the `globalCloudAlpha` multiply
/// (`:251`). Returns the tint normalised to `[0, 1]` — the `render/gore.rs`
/// byte convention.
pub(crate) fn draw_cloud_color(
    sky: [f32; 3],
    scale: f32,
    alpha: f32,
    pass: u8,
    atmo: f32,
    global_cloud_alpha: f32,
) -> [f32; 4] {
    // `Main.ColorOfTheSkies` as XNA bytes (alpha 255 — `SetBackColor` builds
    // it from `new Color(r, g, b)`).
    let bg = [
        (sky[0] * 255.0) as i32 as u8,
        (sky[1] * 255.0) as i32 as u8,
        (sky[2] * 255.0) as i32 as u8,
        255u8,
    ];
    let mut c = crate::cloud::cloud_color(scale, alpha, bg);
    crate::cloud::original_colors_for_cloud(&mut c, scale, pass);
    if atmo < 1.0 {
        // `Color operator *(Color, float)`: `(byte)((int)channel * value)` —
        // XNA truncates, it does not round (`Color.cs` multiply operator).
        for ch in c.iter_mut() {
            *ch = (*ch as f32 * atmo) as i32 as u8;
        }
    }
    let mut out = [0.0f32; 4];
    for (o, ch) in out.iter_mut().zip(c) {
        *o = (ch as f32 * global_cloud_alpha) as i32 as u8 as f32 / 255.0;
    }
    out
}

/// `DrawStar`'s colour chain (`Main.cs:63066-63110`, W4-g69 retire sweep /
/// g69d-star-visual): the greyscale brightness off the SHIMMER-DIMMED sky
/// (`bgColorForStars`), the `starOpacity` (graveyard) multiply, the i%3
/// shimmer channel tint (`twinkle²`, the literal `(1f - num * 0f)` = 1),
/// `Color.Lerp(color, value, shimmerAlpha)` and the final `*= 1 - shimmer`
/// (the `artificial: false` arm; the `artificial: true` shimmer-surface call
/// `:62203` is unported). Returns the RGB bytes, or `None` when the star is
/// skipped (brightness ≤ 0 — `:63072-63074`).
pub(crate) fn draw_star_color(
    star_sky: [f32; 3],
    twinkle: f32,
    star_opacity: f32,
    shimmer_alpha: f32,
    i: usize,
) -> Option<[u8; 3]> {
    // `:63069-63071` — `(int)((255 - ch - 100) * twinkle)` per channel (the
    // fadeIn factor is 1: no SpawnStars(s) respawn in the static field).
    let ch = |c: f32| (255.0 - c * 255.0 - 100.0) * twinkle;
    let n3 = ch(star_sky[0]) as i32;
    let n4 = ch(star_sky[1]) as i32;
    let n3 = (n3 + ch(star_sky[2]) as i32 + n4) / 3;
    if n3 <= 0 {
        return None;
    }
    // `:63076-63080` — `(int)((double)n3 * 1.4)`, capped at 255.
    let v = ((n3 as f64 * 1.4) as i32).min(255);
    // `color *= starOpacity` (`:63086`) — the XNA byte multiply.
    let mut rgb = [(v as f32 * star_opacity) as i32 as u8; 3];
    if shimmer_alpha > 0.0 {
        // `:63090-63101` — i%3: 0 tints G, 1 tints B, 2 tints R.
        let idx = match i % 3 {
            0 => 1,
            1 => 2,
            _ => 0,
        };
        let mut value = rgb;
        // `(float)(int)channel * (1f - num*0f) * twinkle * twinkle` — the
        // literal-1 term, then the two sequential twinkle multiplies.
        value[idx] = (value[idx] as f32 * twinkle * twinkle) as i32 as u8;
        // `Color.Lerp(color, value, num)` (`:63102`) — per-channel byte lerp.
        for k in 0..3 {
            let (a, b) = (rgb[k] as f32, value[k] as f32);
            rgb[k] = (a + (b - a) * shimmer_alpha) as i32 as u8;
        }
        // `color *= 1f - num` (`:63109`, artificial false).
        let f = 1.0 - shimmer_alpha;
        for c in rgb.iter_mut() {
            *c = (*c as f32 * f) as i32 as u8;
        }
    }
    Some(rgb)
}

/// `DrawStarsInBackground`'s heads (`Main.cs:63030-63058`, W4-g69 retire
/// sweep): the graveyard opacity (`1 - intensity * 1.4`, the field dead at
/// ≤ 0), the shimmer sky dim (`colorOfTheSkies *= 1 - shimmerAlpha`, the
/// XNA byte multiply — the colour the `:63051` gate AND `DrawStar` read),
/// and the gate itself: `remixWorld || 255 * (1 - cloudAlpha * atmo) - skyR
/// - 25 > 0` on the RAW cloudAlpha (NOT cloudBGAlpha). The whole pass is
/// skipped at `shimmerAlpha == 1` (`:62086-62088`). Returns
/// `(star_opacity, star_sky, gate_open)`.
pub(crate) fn star_pass_heads(
    tint: [f32; 3],
    graveyard_intensity: f32,
    shimmer_alpha: f32,
    cloud_alpha: f32,
    atmo: f32,
    remix: bool,
) -> (f32, [f32; 3], bool) {
    let star_opacity = if graveyard_intensity > 0.0 {
        (1.0 - graveyard_intensity * 1.4).max(0.0)
    } else {
        1.0
    };
    let star_sky = if shimmer_alpha > 0.0 && shimmer_alpha != 1.0 {
        let f = 1.0 - shimmer_alpha;
        let ch = |v: f32| ((v * 255.0) as i32 as u8 as f32 * f) as i32 as u8 as f32 / 255.0;
        [ch(tint[0]), ch(tint[1]), ch(tint[2])]
    } else {
        tint
    };
    let sky_r = (star_sky[0] * 255.0) as i32 as u8 as i32;
    let gate = star_opacity > 0.0
        && shimmer_alpha != 1.0
        && (remix || 255.0 * (1.0 - cloud_alpha * atmo) - sky_r as f32 - 25.0 > 0.0);
    (star_opacity, star_sky, gate)
}

/// One `DrawClouds_*` pass (`Main.cs:59644-59703`) over the live 200-slot pool:
/// slot order 0..200, the verbatim bucket gate (`cloud_in_bucket` — scale
/// 1.15f passes BOTH 2 and 3, the C# double/float edge), `cY` from the caller
/// (each pass's own formula), the screen X straight off `Cloud.position.X`
/// (`UpdateCloudParallax` already moved it against the camera). `sky` is
/// `Main.ColorOfTheSkies` and `global_cloud_alpha` is
/// `ProcessCloudAlpha() * Main.atmo` (`Main.cs:59330`).
///
/// `EndDrawingClouds` → `NextHorizonRenderer.CloudsEnd`
/// (`NextHorizonRenderer.cs:323-356`) re-draws the buffered sprites through
/// the `GameShaders.Misc["HorizonClouds"]` shader — the sun/moon-tinted
/// cloud shading (the celestial colour + position, the visibility factor
/// scaled by `1 - cloudBGAlpha`). The port's misc-shader infrastructure is
/// unported (W4-g69, documented in ANALYSIS "Clouds and stars"), so the
/// pass draws the byte-chain colours directly — the documented deviation.
#[allow(clippy::too_many_arguments)]
fn draw_cloud_pass(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    field: &crate::cloud::CloudField,
    lookups: &assets::Lookups,
    sky: [f32; 3],
    pass: u8,
    cy_of: impl Fn(&crate::cloud::Cloud) -> f32,
    atmo: f32,
    global_cloud_alpha: f32,
) {
    for c in field.slots.iter() {
        if !c.active || !crate::cloud::cloud_in_bucket(c.scale, pass) {
            continue;
        }
        let Some(tex) = lookups.clouds.get(c.kind.max(0) as usize).and_then(|t| *t) else {
            continue;
        };
        let cy = cy_of(c);
        let color = draw_cloud_color(sky, c.scale, c.alpha, pass, atmo, global_cloud_alpha);
        if color[3] <= 0.0 {
            continue;
        }
        // `DrawData(…, position = (X, cY) + asset.Size()/2, origin =
        // asset.Size()/2, scale, spriteDir)` (`NextHorizonRenderer.cs:244-251`):
        // the UNSCALED half-size offset cancels the origin, so the sprite's
        // top-left lands at `(X, cY)` and it rotates about its scaled centre.
        let w = tex.w * c.scale;
        let h = tex.h * c.scale;
        let uv = if c.flip {
            [tex.uv[2], tex.uv[1], tex.uv[0], tex.uv[3]]
        } else {
            tex.uv
        };
        push_screen_rot(
            r,
            camera,
            zoom,
            screen,
            [c.position.x + w * 0.5, cy + h * 0.5],
            [w, h],
            uv,
            color,
            c.rotation,
        );
    }
}

/// W4-g70: the far-alpha slot of forest set `n` — `UpdateBGVisibility_BackLayer`
/// maps styles 10/11/12 to their own slots (`Main.cs:64145-64209`), set 1 to 0.
fn forest_far_slot(n: usize) -> usize {
    match n {
        0 => 0,
        1 => 10,
        2 => 11,
        _ => 12,
    }
}

/// W4-g70: resolve one live hill id (`forest_hills[n][row]`) to its sheet:
/// the `setBG`-time pair first, then the animated families' packed sheets.
/// `-1` (a style's skipped layer) draws nothing.
fn forest_hill_tex(
    sky: &SkyAssets,
    n: usize,
    row: usize,
    hills: &[[i32; 2]; 4],
) -> Option<BgTex> {
    let id = hills[n][row];
    if id < 0 {
        return None;
    }
    sky.forest_mnt[n][row].or_else(|| sky.forest_mnt_anim.get(&id).copied())
}

/// `DrawSurfaceBG_DrawChangeOverlay` (`Main.cs:61098-61107`): a full-screen
/// `MagicPixel` quad at `Color.Black * flashPower` — the style-change flash.
/// `Color.Transparent` (power 0) skips the draw entirely.
fn draw_change_overlay(r: &mut SpriteRenderer, camera: Vec2, zoom: f32, screen: Vec2, power: f32) {
    if power <= 0.0 {
        return;
    }
    let white = r.white_uv();
    super::bg::push_screen(
        r,
        camera,
        zoom,
        screen,
        [0.0, 0.0],
        [screen.x, screen.y],
        white,
        [0.0, 0.0, 0.0, power],
    );
}

pub(crate) fn draw_sky(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    cloud_field: &crate::cloud::CloudField,
    sky: &SkyAssets,
    // W4-g70 lane 8: the LIVE star pool (`Star.cs`) — the per-tick twinkle/
    // rotation/fadeIn/falling state the draw reads.
    stars: &[crate::star_field::Star],
    surface_px: f32,
    is_day: bool,
    clock_time: f64,
    anim_time: f32,
    sky_uv: Option<[f32; 4]>,
    tint: [f32; 3],
    storm: f32,
    background_enabled: bool,
    // `Main.mapFullscreen` — `ShouldDrawSurfaceBackground` returns false while
    // the fullscreen map is open (`Main.cs:59715-59718`).
    map_fullscreen: bool,
    // `Main.caveParallax` (`Main.cs:53059`, `this.bgParallax = caveParallax`):
    // the "Parallax" slider's scroll factor for the underground backdrop.
    cave_parallax: f32,
    // `Main.bgAlphaFrontLayer` / `Main.bgAlphaFarBackLayer`: per-layer cross-fade
    // alphas; two biome layers blend while `Main.bgStyle` transitions.
    bg_alpha_front: &[f32; 16],
    bg_alpha_far: &[f32; 16],
    light: &LightMap,
    // `Main.hellBlackBoxBottom` (`Main.cs:2463`), threaded to `DrawBackground` —
    // the underworld black box consumes the previous frame's hell floor line.
    hell_black_box_bottom: &mut f32,
    // `Main.shimmerAlpha` (`Main.cs:52800`/`:62086`): the SceneState ramp —
    // 1 skips the underground backdrop AND the natural star field; >0 dims
    // the stars by `1 - alpha` (the artificial pass is g69d's claim).
    shimmer_alpha: f32,
    // The ambient-sky creatures (`AmbientSky`): the live entities and the
    // lookups/context their sheets need. Drawn between the passes at the C#
    // `DrawToDepth` boundaries — see the `DRAW_TO_DEPTH` table below.
    ambience: &[crate::ambience::SkyEntity],
    lookups: &crate::assets::Lookups,
    // W4-f39: the SlimeSky (`SlimeSky : CustomSky`) — the event's own
    // falling-slime background, drawn at the same depth boundaries as the
    // ambient entities (the c39 balloon-band shape). `None` draws nothing.
    slime_sky: Option<&crate::slime_sky::SlimeSky>,
    // `Main.ColorOfTheSkies` (the rain-dimmed sky tint, `Main.cs:63209`) and
    // `Main.atmo` — the entity colour chain (`AmbientSky.cs:63`).
    sky_color: [f32; 3],
    atmo: f32,
    // `Main.GameUpdateCount` (`GetBackgroundCounter`, `Main.cs:61264`) — the
    // animated background sheets' frame counter.
    counter: u32,
    // `Main.mouseTextColor` (`Main.cs:1683`): the Mushroom glow's pulse.
    mouse_text_color: u8,
    // W4-c39 exec (research/114 §1e): the `PartySky` balloon field, drawn at
    // the same `SkyManager` depth boundaries as the ambience
    // (`PartySky.cs:171-245`). The f40 `lantern_sky` shape.
    event_skies: &crate::event_skies::EventSkies,
    // `Main.GraveyardVisualIntensity` (`Main.cs:1327`, the SceneState copy)
    // and `Main.shimmerAlpha` (`:917`) — the `DrawStarsInBackground` head
    // (`Main.cs:63036-63050`): the graveyard dims the star opacity (and
    // kills the field at intensity ≥ 1/1.4), the shimmer dims the star
    // colour. W4-g69 retire sweep (g69d-star-visual).
    graveyard_intensity: f32,
    // `Main.cloudAlpha` RAW (`Main.cs:63051`'s star gate reads
    // `cloudAlpha * atmo`, NOT the storm deck's cloudBGAlpha). W4-g69
    // retire sweep; g70's tip adds the SAME param for the celestial dims —
    // ONE param at the merge (same name, same semantics).
    cloud_alpha: f32,
    // W4-g70: `Main.treeMntBGSet1..4` as `DoUpdate_AnimateBackgrounds` left
    // them this update — the live hill ids of the four forest sets.
    forest_hills: &[[i32; 2]; 4],
    // W4-g70: `WorldGen.BackgroundsCache.GetFlashPower` for the four forest
    // change overlays (areas 0..3, `DrawSurfaceBG_DrawChangeOverlay(0..3)`).
    forest_flash: [f32; 4],
    // W4-g70 lane 8: `SetBackColor`'s `out sunColor`/`out moonColor`
    // (`Main.cs:61971`), threaded to `DrawSunAndMoon` (`:62092`). The moon is
    // white outside a blood moon (`:63917-63920`).
    sun_color: [u8; 3],
    moon_color: [u8; 3],
    // `Main.cloudAlpha` (the raw event alpha, NOT `cloudBGAlpha`) — the
    // celestial dim terms (`Main.cs:62915`/`:62957`) and the star gate.
    // `Main.eclipse` (`Main.cs:1209`) — the Sun3 swap + the horizon kill.
    eclipse: bool,
    // `Main.SmoothedMushroomLightInfluence` (`Main.cs:1283`) — the sun's
    // num12 term and the horizon visibilities.
    mushroom_influence: f32,
    // `Main.player[Main.myPlayer].head` (`Player.cs`): the effective head
    // armour texture id — 12 (Sunglasses) swaps Sun2 in (`Main.cs:62933`).
    player_head_slot: i16,
    // `Main.GraveyardVisualIntensity` — the star-field opacity
    // (`Main.cs:63037-63045`) and the sky's graveyard dim.
    graveyard: f32,
    // `Main.SceneMetrics` + the underground style latch (W4-g72): threaded
    // to `draw_background`'s live style pick (`Main.cs:52823-52841`).
    metrics: &tr_core::scene::SceneMetrics,
    ug_back: &mut super::bg_styles::UgBackState,
    // `Main.shimmerAlpha` — the star dim/recolour (`Main.cs:63047-63050`,
    // `:63087-63110`).
    // W4-c40: the `SkyManager` "Sandstorm" sky (`SandstormSky.cs`) — drawn
    // at the `DrawRemainingDepth` flush slot (`Main.cs:59626`), the same
    // boundary the shallowest ambience entities use. `None` when the sky
    // is inactive (the manager only walks `_activeSkies`).
    sandstorm_sky: Option<&crate::render::sandstorm_sky::SandstormSky>,
    // `Sandstorm.Severity` — the tint quad's `min(1, Severity * 1.5)` term.
    sandstorm_severity: f32,
    // S1-G-13 consumers: skip the underground backdrop tail
    // (`DrawBackground`, `Main.cs:52798`) — the gated layer path renders it
    // into the backgroundTarget via the `RenderBackground` callback
    // (`Main.cs:55200-55218`) instead. The fused default passes `false`.
    skip_underground_background: bool,
    // W4-f40: the `LanternSky` background lantern field, drawn at the same
    // `SkyManager` depth boundaries as the ambience (`LanternSky.cs:154-240`).
    lantern_sky: &crate::render::lantern_sky::LanternSky,
    // `Main.GlobalTimeWrappedHourly` — the lantern glow's hourly bob.
    global_time: f32,
) {
    // `Background_0.png` is the game's own 48x1400 sky gradient: deep blue at the top,
    // pale near the horizon. It is anchored to `Main.worldSurface`.
    const SKY_H: f32 = 1400.0;
    let half = screen * 0.5 / zoom;
    let origin = camera - half;
    let bottom = camera.y + half.y;
    let white = r.white_uv();
    let width = screen.x / zoom;

    let sky_top = surface_px - SKY_H;
    if let Some(uv) = sky_uv {
        let y0 = origin.y.max(sky_top);
        let y1 = bottom.min(surface_px);
        if y1 > y0 {
            let dv = uv[3] - uv[1];
            let v0 = ((y0 - sky_top) / SKY_H).clamp(0.0, 1.0);
            let v1 = ((y1 - sky_top) / SKY_H).clamp(0.0, 1.0);
            let sub = [uv[0], uv[1] + dv * v0, uv[2], uv[1] + dv * v1];
            r.push_back_sprite(
                [origin.x, y0],
                [width, y1 - y0],
                sub,
                [tint[0], tint[1], tint[2], 1.0],
            );
        }
        // Above the gradient's top edge the sky is a flat zenith blue.
        if y0 > origin.y {
            let zenith = [53.0 / 255.0, 43.0 / 255.0, 243.0 / 255.0];
            r.push_back_sprite(
                [origin.x, origin.y],
                [width, y0 - origin.y],
                white,
                [
                    zenith[0] * tint[0],
                    zenith[1] * tint[1],
                    zenith[2] * tint[2],
                    1.0,
                ],
            );
        }
        // Below the average surface level keep the horizon colour; valleys would
        // otherwise show the clear colour under the gradient's bottom edge.
        if bottom > surface_px {
            let horizon = [132.0 / 255.0, 170.0 / 255.0, 248.0 / 255.0];
            r.push_back_sprite(
                [origin.x, surface_px.max(origin.y)],
                [width, bottom - surface_px.max(origin.y)],
                white,
                [
                    horizon[0] * tint[0],
                    horizon[1] * tint[1],
                    horizon[2] * tint[2],
                    1.0,
                ],
            );
        }
    } else {
        r.push_back_sprite(
            [origin.x, origin.y],
            [width, screen.y / zoom],
            white,
            [tint[0], tint[1], tint[2], 1.0],
        );
    }

    // Celestial bodies, stars and the parallax surface background. These are drawn
    // in screen space (compensated for zoom) like `DrawSunAndMoon`/`DrawSurfaceBG`.

    // The vertical anchor of the star/sun plane (`Main.cs:62040`): it slides the
    // whole sky up as the camera descends. `Main.screenPosition` is the
    // *unzoomed* view top-left: the surface background is laid out in that space
    // (`DrawSurfaceBG` uses `screenPosition` directly), so changing
    // `GameZoomTarget` must not move or resize it.
    let world_surface_px = (world.world_surface as f32 * TILE_PX).max(1.0);
    let view_top = camera.y - screen.y * 0.5;
    let bg_top_y = tr_core::sky::sky_bg_top_y(view_top, world.world_surface) as f32;

    // `DrawStarsInBackground` (`Main.cs:63030-63058`) + `DrawStar`
    // (`:63060-63173`), W4-g70 lane 8: the LIVE pool (`crate::star_field`)
    // with the graveyard opacity, the shimmer dim, the fadeIn fade and the
    // falling-star trail. Visibility is driven by the sky colour alone —
    // `255 * (1 - cloudAlpha * atmo) - skyR - 25 <= 0` hides the field —
    // so it fades out across dawn instead of switching off at a fixed hour.
    // The remix world always draws (`:63051`'s first arm).
    let remix_stars = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
    let mut star_opacity = 1.0f32;
    if graveyard > 0.0 {
        let n2 = 1.0 - graveyard * 1.4;
        if n2 <= 0.0 {
            star_opacity = 0.0;
        } else {
            star_opacity *= n2;
        }
    }
    // `:63046-63050`: the shimmer dims the sky colour the brightness reads
    // — XNA's byte multiply (`(byte)(c * 255 * (1 - shimmerAlpha))`).
    let star_sky = if shimmer_alpha > 0.0 {
        sky_color.map(|c| ((c * 255.0) * (1.0 - shimmer_alpha)) as u8)
    } else {
        sky_color.map(|c| (c * 255.0) as u8)
    };
    let sky_r = star_sky[0] as i32;
    // `Main.cs:62086`: the natural star pass skips at shimmerAlpha == 1
    // (the artificial pass at `:62203` draws them instead — no port caller).
    let stars_visible = shimmer_alpha != 1.0
        && star_opacity > 0.0
        && (remix_stars || 255.0 * (1.0 - cloud_alpha * atmo) - sky_r as f32 - 25.0 > 0.0);
    if stars_visible {
        // `:63113-63117`: the tiny-surface worlds pin bgTopY to 0.
        let star_top = if world.world_surface <= 30.0 { 0.0 } else { bg_top_y };
        for (i, s) in stars.iter().enumerate() {
            if s.hidden {
                continue;
            }
            let Some(st) = sky.stars.get(s.tex).and_then(|t| *t) else {
                continue;
            };
            // `DrawStar`'s greyscale brightness (`Main.cs:63068-63080`):
            // `(255 - skyChannel - 100) * twinkle * (1 - fadeIn)` per
            // channel, averaged, ×1.4, capped at 255.
            let num2 = 1.0 - s.fade_in;
            let ch = |c: u8| ((255 - c as i32 - 100) as f32 * s.twinkle * num2) as i32;
            let n3 = ch(star_sky[0]);
            let n4 = ch(star_sky[1]);
            let n3 = (n3 + ch(star_sky[2]) + n4) / 3;
            if n3 <= 0 {
                continue;
            }
            let v = ((n3 as f64 * 1.4) as i32).min(255) as u8;
            // `:63086`: `color *= starOpacity` — XNA's byte multiply.
            let vb = ((v as f32 * star_opacity) as i32) as u8;
            let mut color = [vb, vb, vb, vb];
            // `:63087-63110`: the shimmer recolour — one channel per index
            // class scales by twinkle² (the `1 - num*0` factor is a no-op),
            // lerped by the shimmer alpha; the natural pass then scales by
            // `1 - shimmerAlpha` (the artificial pass at `:62203` is the
            // shimmer overlay's own — no port caller yet).
            if shimmer_alpha > 0.0 {
                let mut value = color;
                let t2 = s.twinkle * s.twinkle;
                match i % 3 {
                    0 => value[1] = (color[1] as f32 * t2) as u8,
                    1 => value[2] = (color[2] as f32 * t2) as u8,
                    _ => value[0] = (color[0] as f32 * t2) as u8,
                }
                color = std::array::from_fn(|c| {
                    (color[c] as f32 + (value[c] as f32 - color[c] as f32) * shimmer_alpha) as u8
                });
                let dim = 1.0 - shimmer_alpha;
                color = color.map(|c| (c as f32 * dim) as u8);
            }
            // `Main.cs:63112-63137`: the design-space position mapped onto
            // the screen and shifted by `sceneArea.bgTopY`; the wrap applies
            // only to NON-falling stars (`:63119`).
            let mut px = s.x / 1920.0 * screen.x;
            let mut py = s.y / 1200.0 * screen.y + star_top;
            if !s.falling {
                if px < 0.0 {
                    px += screen.x;
                }
                if px > screen.x {
                    px -= screen.x;
                }
                if py < 0.0 {
                    py += screen.y;
                }
                if py > screen.y {
                    py -= screen.y;
                }
            }
            let colorf = color.map(|c| c as f32 / 255.0);
            // `:63140-63157`: the falling trail — up to 29 ghosts behind
            // the head at `fallSpeed * j * 0.4`, scale and colour fading
            // by `1 - j/30`.
            if s.falling {
                let n = s.fall_time.min(30);
                for j in 1..n {
                    let jf = j as f32;
                    let fade = 1.0 - jf / 30.0;
                    let k = s.scale * fade;
                    let ghost = color.map(|c| (c as f32 * fade) as u8);
                    push_screen_rot(
                        r,
                        camera,
                        zoom,
                        screen,
                        [px - s.fall_speed_x * jf * 0.4, py - s.fall_speed_y * jf * 0.4],
                        [st.w * k * s.twinkle, st.h * k * s.twinkle],
                        st.uv,
                        ghost.map(|c| c as f32 / 255.0),
                        s.rotation,
                    );
                }
            }
            // `Main.cs:63172`: `tex.Size() * scale * twinkle`, rotated about
            // the centre. The decompile builds the colour from `default`
            // and assigns only R/G/B (`:63083-63086`) — a zero alpha would
            // make every star invisible under the port's blend, so the
            // port draws them at the computed brightness with the byte
            // alpha the `color *=` chain produces (see ANALYSIS.md).
            let k = s.scale * s.twinkle;
            push_screen_rot(
                r,
                camera,
                zoom,
                screen,
                [px, py],
                [st.w * k, st.h * k],
                st.uv,
                colorf,
                s.rotation,
            );
        }
    }

    // `DrawSunAndMoon` (`Main.cs:62848-63028`): the sun and the moon ride the
    // same arc, anchored to `Main.bgTopY` (`Main.cs:62040`) and drawn only
    // while the camera is near the surface (`Main.cs:62090`). The original
    // computes both bodies every frame but draws only the one matching
    // `Main.dayTime`. W4-g70 lane 8: the event bodies (eclipse Sun3, the
    // Sunglasses Sun2 + Extra_280 overlay, the Smiley/Pumpkin/Snow moons),
    // the byte-exact colour chain and the `DrawSun` flare quads.
    if view_top / TILE_PX < world.world_surface as f32 + 2.0 {
        let bg_top_y = tr_core::sky::sky_bg_top_y(view_top, world.world_surface);
        let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
        let drunk = world.seeds.contains(tr_core::seeds::SeedFlags::DRUNK);
        if is_day && !remix {
            // `:62910-62913`: the remix world draws no sun in-game.
            if let Some(sun_tex) = sky.sun {
                let mut body = tr_core::sky::celestial_body(
                    clock_time,
                    tr_core::sky::DAY_LENGTH,
                    bg_top_y,
                    sun_tex.w,
                    screen.x,
                );
                // `num5 *= 1.1f` in the day branch (`Main.cs:62914`).
                body.scale *= 1.1;
                // `num12 = 1f - tempMushroomInfluence - cloudAlpha * 1.5f *
                // atmo`, floored at 0 (`:62915-62919`).
                let num12 = (1.0 - mushroom_influence - cloud_alpha * 1.5 * atmo).max(0.0);
                let byte = |v: f32| (v as i32) as u8;
                // `:62920-62921`: the two colour passes. `color`'s R is the
                // flat 255*num12; `color2` rides sunColor with alpha = B.
                let mut color = [
                    byte(255.0 * num12),
                    byte(sun_color[1] as f32 * num12),
                    byte(sun_color[2] as f32 * num12),
                    byte(255.0 * num12),
                ];
                let mut color2 = [
                    byte(sun_color[0] as f32 * num12),
                    byte(sun_color[1] as f32 * num12),
                    byte(sun_color[2] as f32 * num12),
                    byte(sun_color[2] as f32 * num12),
                ];
                // `:62922-62938`: the plain sun is white; the eclipse swaps
                // Sun3 in, the Sunglasses head (head == 12) swaps Sun2 +
                // the Extra_280 overlay. `flag` (either swap) drops the
                // second pass's alpha to (B-60)*num12 (`:62939-62942`).
                let mut tex = sun_tex;
                let mut flag = false;
                let mut sunglasses = false;
                if eclipse {
                    if let Some(t) = sky.sun_eclipse {
                        tex = t;
                    }
                    flag = true;
                } else {
                    color = [255, 255, 255, 255];
                    if player_head_slot == 12 {
                        if let Some(t) = sky.sun_sunglasses {
                            tex = t;
                        }
                        sunglasses = true;
                        flag = true;
                    }
                }
                if flag {
                    color2[3] = byte((sun_color[2] as f32 - 60.0) * num12);
                }
                let size = [tex.w * body.scale, tex.h * body.scale];
                let to_f = |c: [u8; 4]| c.map(|v| v as f32 / 255.0);
                push_screen_rot(
                    r, camera, zoom, screen,
                    [body.x, body.y], size, tex.uv, to_f(color), body.rotation,
                );
                push_screen_rot(
                    r, camera, zoom, screen,
                    [body.x, body.y], size, tex.uv, to_f(color2), body.rotation,
                );
                // `HorizonRenderer.DrawSun` (`Main.cs:62947`,
                // `NextHorizonRenderer.cs:168-181`): five flare quads at the
                // sun position — flare1 ×3/×2 at the sunset strength, flare2
                // ×2/×1.5/×1 at the sunrise strength, all scaled by the
                // celestial power. The colour is (255,255,255,0) × strength:
                // under the premultiplied One/OneMinusSrcAlpha blend the
                // alpha-0 tint leaves the destination alpha untouched while
                // the tinted texture rgb ADDS — an additive glow, exactly
                // what the port's shader does with the same factors
                // (sprite.wgsl's documented convention).
                let (mut sunset, mut sunrise) = horizon_visibilities(
                    clock_time, true, atmo, cloud_alpha, mushroom_influence, eclipse,
                );
                let celestial = celestial_effect_power(clock_time, true);
                sunset *= celestial;
                sunrise *= celestial;
                if let Some(f1) = sky.sunflare1 {
                    for (scale, strength) in [(3.0f32, sunset * 0.75), (2.0, sunset * 0.35)] {
                        push_screen_rot(
                            r, camera, zoom, screen,
                            [body.x, body.y],
                            [f1.w * scale, f1.h * scale],
                            f1.uv,
                            [strength, strength, strength, 0.0],
                            0.0,
                        );
                    }
                }
                if let Some(f2) = sky.sunflare2 {
                    for (scale, strength) in
                        [(2.0f32, sunrise * 0.7 * 0.5), (1.5, sunrise * 0.3 * 0.5), (1.0, sunrise * 0.2 * 0.5)]
                    {
                        push_screen_rot(
                            r, camera, zoom, screen,
                            [body.x, body.y],
                            [f2.w * scale, f2.h * scale],
                            f2.uv,
                            [strength, strength, strength, 0.0],
                            0.0,
                        );
                    }
                }
                // `:62948-62953`: the sunglasses overlay at alpha 100.
                if sunglasses {
                    if let Some(ov) = sky.sunglasses_overlay {
                        push_screen_rot(
                            r, camera, zoom, screen,
                            [body.x, body.y], size, ov.uv,
                            [
                                color[0] as f32 / 255.0,
                                color[1] as f32 / 255.0,
                                color[2] as f32 / 255.0,
                                100.0 / 255.0,
                            ],
                            body.rotation,
                        );
                    }
                }
            }
        }
        if !is_day {
            // `num13 = 1f - cloudAlpha * 1.5f * atmo` (`:62957-62961`);
            // `moonColor *= num13` is XNA's byte multiply (`:62962`). The
            // moon colour is white outside a blood moon (`SetBackColor`'s
            // `:63917-63920` reset, threaded in as `moon_color`).
            let num13 = (1.0 - cloud_alpha * 1.5 * atmo).max(0.0);
            let mc = moon_color.map(|c| ((c as f32 * num13) as i32) as u8);
            // `moonColor *= num13` scales the ALPHA too (XNA's Color*float
            // multiplies all four channels; the input alpha is 255).
            let mc4 = [
                mc[0] as f32 / 255.0,
                mc[1] as f32 / 255.0,
                mc[2] as f32 / 255.0,
                ((255.0 * num13) as i32) as f32 / 255.0,
            ];
            // The body geometry always uses `Moon[moonType]`'s width
            // (`num6`, `:62862`) — the event sheets share the 50 px row.
            let moon_w = sky.moon[world.moon_type.min(8) as usize].map_or(50.0, |m| m.w);
            let body = tr_core::sky::celestial_body(
                clock_time,
                tr_core::sky::NIGHT_LENGTH,
                bg_top_y,
                moon_w,
                screen.x,
            );
            // `new Rectangle(0, width * moonPhase, width, width)`: one phase
            // row (`:62970-62978`). The Smiley is a single whole frame at
            // rotation `num9/2 + π` (`:62964-62967`).
            let phase_row = |m: BgTex| {
                let phase = world.moon_phase.min(7) as f32;
                let dv = (m.uv[3] - m.uv[1]) / 8.0;
                let v0 = m.uv[1] + dv * phase;
                [m.uv[0], v0, m.uv[2], v0 + dv]
            };
            let size = [moon_w * body.scale, moon_w * body.scale];
            if drunk {
                if let Some(m) = sky.moon_smiley {
                    push_screen_rot(
                        r, camera, zoom, screen,
                        [body.x, body.y],
                        [m.w * body.scale, m.h * body.scale],
                        m.uv,
                        mc4,
                        body.rotation / 2.0 + std::f32::consts::PI,
                    );
                }
            } else if world.pumpkin_moon {
                if let Some(m) = sky.moon_pumpkin {
                    push_screen_rot(
                        r, camera, zoom, screen,
                        [body.x, body.y], size, phase_row(m), mc4, body.rotation,
                    );
                }
            } else if world.snow_moon {
                if let Some(m) = sky.moon_snow {
                    push_screen_rot(
                        r, camera, zoom, screen,
                        [body.x, body.y], size, phase_row(m), mc4, body.rotation,
                    );
                }
            } else if let Some(m) = sky.moon[world.moon_type.min(8) as usize] {
                push_screen_rot(
                    r, camera, zoom, screen,
                    [body.x, body.y], size, phase_row(m), mc4, body.rotation,
                );
            }
        }
    }

    // `DrawSurfaceBG_BackMountainsStep1/2` + the front biome chains —
    // extracted so the capture's background pass renders the SAME chains
    // at the chunk camera with the capture-only bg-alpha arrays
    // (`Main.DrawCapture`'s `UpdateBGVisibility_*` override,
    // `Main.cs:56675-56680`; S1-G-13 consumers). The `SkyManager` depth
    // tracker threads through the call so the `DrawRemainingDepth` flush
    // below sees the last boundary the chains emitted.
    let mut amb_last = f32::INFINITY;
    let amb_ctx = crate::ambience::SkyDrawCtx {
        camera,
        zoom,
        screen,
        color_of_the_skies: sky_color,
        atmo,
        bg_alpha_front,
        background_enabled,
        world_surface: world.world_surface,
    };
    draw_surface_bg(
        r,
        camera,
        zoom,
        screen,
        world,
        sky,
        cloud_field,
        surface_px,
        sky_color,
        atmo,
        map_fullscreen,
        slime_sky,
        event_skies,
        forest_hills,
        forest_flash,
        sandstorm_sky,
        sandstorm_severity,
        light,
        cave_parallax,
        hell_black_box_bottom,
        metrics,
        ug_back,
        shimmer_alpha,
        lantern_sky,
        global_time,
        bg_alpha_front,
        bg_alpha_far,
        tint,
        storm,
        background_enabled,
        anim_time,
        counter,
        mouse_text_color,
        ambience,
        lookups,
        &amb_ctx,
        &mut amb_last,
    );

    // `Main.cs:59626` `DrawRemainingDepth`: the shallowest entities (below the
    // last front-row boundary) draw after every surface layer, before the
    // underground backdrop.
    crate::ambience::draw_range_remaining(r, ambience, lookups, &amb_ctx, &mut amb_last);
    // `Main.DrawBackground_SurfaceTransitionBackground`,
    // `Main.DrawBackground_DirtBackground`, `DrawBackground_DrawUnderworldBlackBox`,
    // `DrawBackground_DrawRockLayer` and `DrawBackground_DrawMagmaLayer`: the textured
    // underground backdrop, anchored to `worldSurface` across the whole screen, with
    // the rock bands, the animated magma and the underworld black box below it. (A
    // per-column gradient used to hang a dark "wall" underneath every player-placed
    // row of blocks.)
    if !crate::env::no_bg() && !skip_underground_background {
        draw_background(
            r,
            camera,
            zoom,
            screen,
            world,
            sky,
            light,
            cave_parallax,
            anim_time,
            hell_black_box_bottom,
            // `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023`): the
            // port's `TR_SHOW_INVISIBLE` A/B pin (false in normal play).
            crate::env::show_invisible(),
            metrics,
            ug_back,
            shimmer_alpha,
        );
    }
}

/// `Main.DrawSurfaceBG` (`Main.cs:59302-60290` + the per-biome chains
/// `:59986-61370`): the storm banks, the far/near mountain steps, the three
/// cloud passes and the front biome tree lines. Extracted verbatim from
/// [`draw_sky`] so the capture's background pass (`Main.DrawCapture`'s
/// `DrawSurfaceBG` call, `Main.cs:56837`) renders the SAME chains at the
/// chunk camera with the capture-only bg-alpha arrays (S1-G-13 consumers).
/// `amb_ctx`/`amb_last` are the `SkyManager` depth tracker the caller
/// threads (the flush stays at the caller's `DrawRemainingDepth` site).
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_surface_bg(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    sky: &SkyAssets,
    // Union: the full W4 g39/f39/c39/f40/c40/g69/g70/g72 surface-bg body
    // needs the draw-sky inputs the extraction splice dropped.
    cloud_field: &crate::cloud::CloudField,
    surface_px: f32,
    sky_color: [f32; 3],
    atmo: f32,
    map_fullscreen: bool,
    slime_sky: Option<&crate::slime_sky::SlimeSky>,
    event_skies: &crate::event_skies::EventSkies,
    forest_hills: &[[i32; 2]; 4],
    forest_flash: [f32; 4],
    sandstorm_sky: Option<&crate::render::sandstorm_sky::SandstormSky>,
    sandstorm_severity: f32,
    light: &LightMap,
    cave_parallax: f32,
    hell_black_box_bottom: &mut f32,
    metrics: &tr_core::scene::SceneMetrics,
    ug_back: &mut super::bg_styles::UgBackState,
    shimmer_alpha: f32,
    lantern_sky: &crate::render::lantern_sky::LanternSky,
    global_time: f32,
    bg_alpha_front: &[f32; 16],
    bg_alpha_far: &[f32; 16],
    tint: [f32; 3],
    storm: f32,
    background_enabled: bool,
    anim_time: f32,
    counter: u32,
    mouse_text_color: u8,
    ambience: &[crate::ambience::SkyEntity],
    lookups: &crate::assets::Lookups,
    amb_ctx: &crate::ambience::SkyDrawCtx<'_>,
    amb_last: &mut f32,
) {
    let world_surface_px = (world.world_surface as f32 * TILE_PX).max(1.0);
    let view_top = camera.y - screen.y * 0.5;
    let origin = camera - screen * 0.5 / zoom;
    // `Main.bgAlphaFrontLayer`/`bgAlphaFarBackLayer`: each biome layer is drawn with
    // `ColorOfSurfaceBackgroundsBase * alpha`, so two biomes blend while the style
    // cross-fades (`backgroundLayerTransitionSpeed = 0.05`). Index tables:
    //   front: forest 0, desert 2, ocean 4, snow 7
    //   far  : forest 0, desert 1, ocean 3, snow 4     (`UpdateBGVisibility_BackLayer`)
    let front_forest = bg_alpha_front[0];
    let front_desert = bg_alpha_front[2];
    let front_snow = bg_alpha_front[7];
    let far_forest = bg_alpha_far[0];
    // `UpdateBGVisibility_BackLayer`: desert far = 1, ocean = 3, snow = 4.
    let far_desert = bg_alpha_far[1];
    let far_snow = bg_alpha_far[4];
    let far_ocean = bg_alpha_far[3];
    // `Main.cs:59327`: `num3 = (0 - (screenPosition.Y - 300)) / (worldSurface * 16)`.
    let magic = (300.0 - view_top) / world_surface_px;
    let sc_adj = bg_scale_adjust(world, view_top, screen.y);
    // `Main.DrawSurfaceBG`: `pushBGTopHack = 30` for the mountain passes, then
    // `-180` after the horizon layer, so the front layers use `30 - 180 = -150`.
    // The `+30`/`-150` are folded into the layer tops below.
    // The block from `Main.cs:59354` (`screenPosition.Y / 16 <= worldSurface + 10`)
    // carries the storm banks, the mountain steps and the two closer cloud passes;
    // the distant clouds gate on the tighter `+16 px` sky check of
    // `DrawClouds_Distant` (`Main.cs:59647`) and the front layers run on
    // `Main.BackgroundEnabled` alone (`Main.cs:59527-59531`).
    let bg_block = view_top <= world_surface_px + 10.0 * TILE_PX;
    // `Main.BackgroundEnabled`: the parallax layers only; the gradient and the
    // clouds above stay, exactly like the original toggle.
    let show_bg = background_enabled && bg_block;
    // W4-f40: the `LanternSky` rides the SAME `SkyManager` depth tracker as
    // the ambience — every `DrawToDepth(minDepth)` boundary draws its
    // `[minDepth, last)` band (`LanternSky.cs:169-187`'s own cull). The
    // tracker mirrors `amb_last`; the final flush draws the remainder.
    let mut lantern_last = f32::INFINITY;
    let draw_lantern_band = |r: &mut SpriteRenderer, min_depth: f32, last: &mut f32| {
        if *last <= min_depth {
            return;
        }
        lantern_sky.draw(
            r,
            lookups,
            camera,
            zoom,
            screen,
            world.world_surface,
            global_time,
            min_depth,
            *last,
        );
        *last = min_depth;
    };
    // W4-c39 exec (research/114 §1e): the `PartySky` rides the SAME
    // `SkyManager` depth tracker as the ambience — every `DrawToDepth(min)`
    // boundary draws its `[min, last)` band (`PartySky.cs:185-198`'s own
    // cull). The tracker mirrors `amb_last`; the final flush draws the
    // remainder. The f40 `lantern_sky` shape.
    let mut party_last = f32::INFINITY;
    let draw_party_band = |r: &mut SpriteRenderer, min_depth: f32, last: &mut f32| {
        if *last <= min_depth {
            return;
        }
        crate::event_skies::draw_balloons_band(
            r,
            event_skies,
            lookups,
            camera,
            zoom,
            screen,
            sky_color,
            surface_px,
            min_depth,
            *last,
        );
        *last = min_depth;
    };
    // `DrawSurfaceBG`'s three cloud passes, in the original order: distant
    // (`Main.cs:59334`, before the storm banks), closer (`Main.cs:59449`,
    // between the mountain steps) and closest (`Main.cs:59520`, before the front
    // layers). The live pool's screen X is `Cloud.position.X` itself —
    // `Cloud.UpdateCloudParallax` (`Cloud.cs:295-316`, called per draw at
    // `Main.cs:61833`) already shifted it against the camera pan and
    // `Cloud.Update` integrates the wind drift (`Cloud.cs:391`).
    // `num5 = ProcessCloudAlpha() * Main.atmo` (`Main.cs:59330`): no custom
    // skies are active in-world, so `ProcessCloudAlpha` clamps 1
    // (`SkyManager.cs` `ProcessCloudAlpha`) and the factor is `atmo` itself.
    let global_cloud_alpha = atmo;
    // `flag = ShouldDrawSurfaceBackground()` (`Main.cs:59300`, `:59707-59720`):
    // false on remix worlds in-game (the drunk+remix gen combo aside) and
    // while the fullscreen map is open (`Main.cs:59715`).
    let should_draw_surface =
        !world.seeds.contains(tr_core::seeds::SeedFlags::REMIX) && !map_fullscreen;
    // The closer/closest block's own gate (`Main.cs:59354`): `!drunkWorldGen &&
    // flag && screenPosition.Y / 16 <= worldSurface + 10`; each pass then
    // re-checks the tighter `screenPosition.Y < worldSurface * 16 + 16`
    // (`Main.cs:59648`, `:59667`).
    let cloud_block = should_draw_surface
        && !world.seeds.contains(tr_core::seeds::SeedFlags::DRUNK)
        && bg_block
        && view_top < world_surface_px + TILE_PX;
    if should_draw_surface && view_top < world_surface_px + TILE_PX {
        // `DrawClouds_Distant` (`Main.cs:59682-59703`): unscaled Y, hack 0.
        draw_cloud_pass(
            r,
            camera,
            zoom,
            screen,
            cloud_field,
            lookups,
            sky_color,
            1,
            |c| cloud_distant_cy(c.position.y, magic, sc_adj),
            atmo,
            global_cloud_alpha,
        );
    }
    // `DrawSurfaceBG` positions every layer from `screenPosition.Y` alone; anchoring
    // them to the surface of the camera's own column made the whole skyline jump
    // whenever the camera crossed a hill, so the horizon follows the world's average
    // surface (a constant) and moves only with the camera.
    if show_bg && storm > 0.01 {
        // `cloudBG` layers: two storm cloud banks behind the mountains, fading in
        // with cloudAlpha. They keep the sky busy between the horizon and the zenith.
        if let Some(t) = sky.storm_far {
            draw_bg_layer(
                r,
                camera,
                zoom,
                screen,
                &t,
                1.65,
                0.09,
                magic * 900.0 + 600.0 + sc_adj + 30.0,
                false,
                [tint[0] * 0.9, tint[1] * 0.92, tint[2] * 0.96],
                storm,
            );
        }
        if let Some(t) = sky.storm_near {
            draw_bg_layer(
                r,
                camera,
                zoom,
                screen,
                &t,
                1.85,
                0.12,
                magic * 1100.0 + 750.0 + sc_adj + 30.0,
                false,
                tint,
                (storm * 1.5).min(1.0),
            );
        }
    }
    // Layers are positioned like `DrawSurfaceBG` but drawn in screen pixels (the
    // original applies the zoom in its sprite-batch transform). Keep the bottom of
    // every layer below the local horizon so no sky leaks between it and the terrain.
    //
    // `DrawSurfaceBG_BackMountainsStep1`: far mountains, parallax 0.15, scale 2.0,
    // each biome at its `bgAlphaFarBackLayer` value. Vanilla order is 0, 1 (desert),
    // then 4 (snow). `Background_22/23` are skipped for `desertBG == 4` (the
    // oasis style, `Main.cs:59850`/`:59906`); the snow mountains come from the
    // world's `setBG(3)` style and carry their `GetLayerYOffset`.
    // Step1's first act is `DrawToDepth(1f/0.15)` (`Main.cs:59812`): the
    // closest entities (EOC slices above depth 6.667) draw behind the
    // mountains.
    // W4-f39: the SlimeSky's slimes ride the same boundary (the C#
    // `SkyManager.DrawDepthRange` walks every active sky; the band is
    // (min, last] BEFORE the tracker moves — the activation order draws
    // Ambience first, Slime second).
    let band_max = *amb_last;
    crate::ambience::draw_range_min(
        r,
        ambience,
        lookups,
        amb_ctx,
        1.0 / 0.15,
        amb_last,
    );
    // W4-c39: the party balloons' far band rides the same boundary.
    draw_party_band(r, 1.0 / 0.15, &mut party_last);
    if let Some(ss) = slime_sky {
        ss.draw_band(
            r,
            &sky.slime_sky,
            camera,
            zoom,
            screen,
            origin,
            sky_color,
            1.0 / 0.15,
            band_max,
        );
    }
    draw_lantern_band(r, 1.0 / 0.15, &mut lantern_last);
    if show_bg {
        // W4-g70: the four forest far hills (`Main.cs:59827-59846`), in the
        // C#'s order 0, 10, 11, 12 — each at its own `bgAlphaFarBackLayer`
        // slot, id from the live animated set, `GetLayerYOffset` nudged.
        // Step1's own `screenPosition.Y < worldSurface*16 + 16` gate
        // (`:59823`) is tighter than the enclosing +10-tile block.
        let hills_visible = view_top < world_surface_px + TILE_PX;
        for n in 0..4 {
            let alpha = bg_alpha_far[forest_far_slot(n)];
            if hills_visible && alpha > 0.0 {
                if let Some(far) = forest_hill_tex(sky, n, 0, forest_hills) {
                    draw_bg_layer(
                        r,
                        camera,
                        zoom,
                        screen,
                        &far,
                        2.0,
                        0.15,
                        magic * 1300.0 + 1090.0 + sc_adj + 30.0
                            + layer_y_offset(forest_hills[n][0]),
                        false,
                        tint,
                        alpha,
                    );
                }
            }
        }
        // `DrawSurfaceBG_BackMountainsStep1` desert row (`Main.cs:59847-59854`):
        // `Background_23` is skipped when `WorldGen.desertBG == 4` (the oasis
        // style brings its own far rows).
        if far_desert > 0.0 && world.desert_bg_style != 4 {
            if let Some(far) = sky.desert_far_mnt {
                draw_bg_layer(
                    r,
                    camera,
                    zoom,
                    screen,
                    &far,
                    2.0,
                    0.15,
                    magic * 1300.0 + 1090.0 + sc_adj + 30.0,
                    false,
                    tint,
                    far_desert,
                );
            }
        }
        if far_snow > 0.0 {
            if let Some(far) = sky.snow_far {
                draw_bg_layer(
                    r,
                    camera,
                    zoom,
                    screen,
                    &far,
                    2.0,
                    0.15,
                    magic * 1300.0 + 1090.0 + sc_adj + 30.0
                        + layer_y_offset(world.snow_mountains()[0]),
                    false,
                    tint,
                    far_snow,
                );
            }
        }
        // Step1's corrupt-desert (24), crimson (24 unless crimsonBG == 5) and
        // hallow-3 (246) rows (`Main.cs:59855-59877`) ride the g70 lane's
        // far-layer slots 2/5/6; the port's cross-fade never targets them yet.
    }

    // `DrawClouds_Closer` (`Main.cs:59663-59680`): `cY = y * (screenHeight / 600)
    // + bgTopY + 150`. `bgTopY` here is whichever value the last writer left: the
    // mountain Step1 when parallax is on (`Main.cs:59808`), else the plain
    // `bgTopY` of `Main.cs:59328` — the storm banks' own `bgTopY`
    // (`Main.cs:59433`) is overwritten by Step1 before this pass runs.
    // Step1's closing `DrawToDepth(5f)` (`Main.cs:59878`) runs before this
    // pass: the mid-depth entities draw between the mountain rows and the
    // closer clouds.
    let band_max = *amb_last;
    crate::ambience::draw_range_min(r, ambience, lookups, &amb_ctx, 5.0, amb_last);
    // W4-c39: the party balloons' mid band rides the same boundary.
    draw_party_band(r, 5.0, &mut party_last);
    if let Some(ss) = slime_sky {
        ss.draw_band(
            r,
            &sky.slime_sky,
            camera,
            zoom,
            screen,
            origin,
            sky_color,
            5.0,
            band_max,
        );
    }
    if cloud_block {
        // `bgTopY` as Step1 (`Main.cs:59808`) or the plain head (`:59328`)
        // leaves it; the pass adds its own `-50 + 200`.
        let closer_bg_top = if background_enabled {
            magic * 1300.0 + 1090.0 + sc_adj + 30.0
        } else {
            magic * 1200.0 + 1190.0 + sc_adj
        };
        draw_cloud_pass(
            r,
            camera,
            zoom,
            screen,
            cloud_field,
            lookups,
            sky_color,
            2,
            |c| cloud_closer_cy(c.position.y, screen.y, closer_bg_top),
            atmo,
            global_cloud_alpha,
        );
    }

    // `DrawSurfaceBG_BackMountainsStep2`: near mountains, parallax 0.20, scale 2.3,
    // in vanilla order: forest set 1 (0), desert (1), ocean (`Main.oceanBG`
    // from `setBG(7)`, 3), snow (4, `Main.snowMntBG[1]`), then forest sets
    // 2-4 (10/11/12) — the C# walks 0..6 first and the three extra forest
    // slots last (`Main.cs:59895-59962`).
    if show_bg {
        // Step2's own `+16 px` gate (`Main.cs:59891`), like Step1's.
        let hills_visible = view_top < world_surface_px + TILE_PX;
        if hills_visible && far_forest > 0.0 {
            if let Some(near) = forest_hill_tex(sky, 0, 1, forest_hills) {
                draw_bg_layer(
                    r,
                    camera,
                    zoom,
                    screen,
                    &near,
                    2.3,
                    0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0
                        + layer_y_offset(forest_hills[0][1]),
                    false,
                    tint,
                    far_forest,
                );
            }
        }
        // Step2's desert row skips `Background_22` for desertBG == 4
        // (`Main.cs:59903-59910`), like Step1's 23.
        if far_desert > 0.0 && world.desert_bg_style != 4 {
            if let Some(near) = sky.desert_near_mnt {
                draw_bg_layer(
                    r,
                    camera,
                    zoom,
                    screen,
                    &near,
                    2.3,
                    0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0,
                    false,
                    tint,
                    far_desert,
                );
            }
        }
        if far_ocean > 0.0 {
            if let Some(sea) = sky.ocean {
                // `DrawSurfaceBG_DrawBackMountainsLayer(Main.oceanBG)`
                // (`Main.cs:59916-59920`): the ocean sheet's `GetLayerYOffset`
                // (283: -800, 332/340: -25) shifts its top like every other
                // Step2 row.
                draw_bg_layer(
                    r,
                    camera,
                    zoom,
                    screen,
                    &sea,
                    2.3,
                    0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0
                        + layer_y_offset(world.ocean_bg()),
                    false,
                    tint,
                    far_ocean,
                );
            }
        }
        if far_snow > 0.0 {
            if let Some(near) = sky.snow_near {
                draw_bg_layer(
                    r,
                    camera,
                    zoom,
                    screen,
                    &near,
                    2.3,
                    0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0
                        + layer_y_offset(world.snow_mountains()[1]),
                    false,
                    tint,
                    far_snow,
                );
            }
        }
        // W4-g70: forest sets 2-4's near hills (`Main.cs:59939-59962`), each
        // gated on its own far alpha and `treeMntBGSetN[1] > -1`.
        for n in 1..4 {
            let alpha = bg_alpha_far[forest_far_slot(n)];
            if hills_visible && alpha > 0.0 && forest_hills[n][1] > -1 {
                if let Some(near) = forest_hill_tex(sky, n, 1, forest_hills) {
                    draw_bg_layer(
                        r,
                        camera,
                        zoom,
                        screen,
                        &near,
                        2.3,
                        0.20,
                        magic * 1400.0 + 1260.0 + sc_adj + 30.0
                            + layer_y_offset(forest_hills[n][1]),
                        false,
                        tint,
                        alpha,
                    );
                }
            }
        }
    }

    // `DrawClouds_Closest` (`Main.cs:59644-59661`): `cY = y * (screenHeight /
    // 600) - 100 + bgTopY * 1.01 - 150`, with `bgTopY` from the near-mountain
    // setup (`Main.cs:59514`; the port always has the forest near layer) or,
    // with parallax off, the plain `bgTopY` of `Main.cs:59328`.
    if cloud_block {
        let closest_bg_top = if background_enabled {
            magic * 1400.0 + 1260.0 + sc_adj + 30.0
        } else {
            magic * 1200.0 + 1190.0 + sc_adj
        };
        draw_cloud_pass(
            r,
            camera,
            zoom,
            screen,
            cloud_field,
            lookups,
            sky_color,
            3,
            |c| cloud_closest_cy(c.position.y, screen.y, closest_bg_top),
            atmo,
            global_cloud_alpha,
        );
    }

    // Front layer loop (`Main.DrawSurfaceBG` 59529): forest (0), desert (2), snow (7).
    // Each layer draws with `ColorOfSurfaceBackgroundsBase * bgAlphaFrontLayer[i]`.
    // Each active biome's row chain emits its own `DrawToDepth(1f/parallax)`
    // boundaries (`Main.cs:61304+`); the pass threads the tracker so a second
    // cross-fading biome's boundaries no-op like the original's.
    //
    // W4-g72 closeout SURFACE-OWNERSHIP cross-note (g71 owns the surface
    // draw-tables): the five v16-owned variants below (corrupt/jungle/hallow/
    // crimson/mushroom) read `world.*_bg()` → `surface_bg_texture_ids`
    // (`WorldGen.setBG` slots 1/2/4/5/8) — the g71-shape style→ids expansion,
    // already live. The forest/desert/snow/ocean chains are STYLE-0 PINNED
    // here (`[50,51,52]` / `[21,20,-1]` / `[37,38,39]` / `Background_28`):
    // their style tables (`SetForestBGSet` slots 0/10/11/12, `setBG` 3/6/7)
    // are g70/g71's to port — g71's draw-side reads `world.forest_bg(n)` /
    // `desert_bg_set()[0]` / `snow_bg()` / `ocean_bg()` (its sky.rs), so this
    // lane does NOT re-port them; the pins lift to those reads at the merge.
    // See docs/research/64c + ANALYSIS "The surface-bg draw ownership".
    let mut amb_pass = show_bg.then_some(crate::render::bg::AmbiencePass {
        entities: ambience,
        lookups,
        ctx: &amb_ctx,
        last: amb_last,
        // W4-c39: the party sky's band state threads the front-row chain
        // (the f40 `lantern_sky`/`lantern_last` shape).
        event_skies,
        party_last: &mut party_last,
        party_surface_px: surface_px,
        slime_sky,
        lantern_sky,
        lantern_last: &mut lantern_last,
        global_time,
    });
    if show_bg {
        // `pushBGTopHack` for the front loop is `30 - 180` (`Main.cs:59302`,
        // `:59346`, `:59525`): every `top` below is the C# constant minus 150.
        const HACK: f32 = -150.0;
        // Dispatch order is the C# `l` loop's (`Main.cs:59529-59597`).

        // `DrawSurfaceBG_Forest` (`Main.cs:61273-61519`) for tree set 1
        // (`Main.treeBGSet1`, `World.forest_bg_styles[0]`): the row bases,
        // per-id overrides, top rewrites/nudges and the 51/54 mirror ride
        // `forest_layers`; the 173 row swaps its drawn sheet to the
        // `251 + counter/10 % 4` frame (`:61373`).
        let forest_ids = world.forest_bg(0).1;
        let frame_173 = sky.forest_173_frames[(forest_173_frame_id(counter) - 251) as usize];
        let forest = forest_layers(forest_ids, sky.forest_sets[0], HACK, frame_173);
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &forest,
            front_forest,
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            false,
        );
        // `DrawSurfaceBG_DrawChangeOverlay(0)` (`Main.cs:59537`).
        draw_change_overlay(r, camera, zoom, screen, forest_flash[0]);

        // `DrawSurfaceBG_Corrupt` (`Main.cs:61109-61218`). Row 1 is mirrored; the
        // `ids[0] == 56` nudge (`:61128`, `:61199`) applies to every row and reads
        // row 0's id, not the row's own.
        let corrupt_ids = world.corrupt_bg();
        let corrupt = front_layers(
            corrupt_ids,
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            sky.corrupt_layers,
            HACK,
        );
        let nudge = if corrupt_ids[0] == 56 { -100.0 } else { 0.0 };
        let corrupt: [FrontLayer; 3] = std::array::from_fn(|i| FrontLayer {
            top: corrupt[i].top + nudge,
            flip: i == 1,
            ..corrupt[i]
        });
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &corrupt,
            bg_alpha_front[1],
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            false,
        );

        // `DrawSurfaceBG_Desert` (`Main.cs:61019-61096`) on
        // `Main.desertBackgroundSet.Pure.Backgrounds`: rows gated on `id > 0`,
        // parallax 0.37/0.49, scale 1.25/1.34, tops 1750/2150 — the oasis
        // style 4 adds a third row (250) and `SetBackgroundOffsets` replaces
        // all three rows' magic numbers (`:60444-60461`).
        let desert_ids = world.desert_bg_set()[0];
        let desert = front_layers(
            desert_ids,
            [
                (1.25, 0.37, 1800.0, 1750.0),
                (1.34, 0.49, 2100.0, 2150.0),
                (1.34, 0.49, 2100.0, 2150.0),
            ],
            [sky.desert_far, sky.desert_near, sky.desert_third],
            HACK,
        );
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &desert,
            front_desert,
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            false,
        );

        // `DrawSurfaceBG_Jungle` (`Main.cs:60928-61017`): `GetLayerYOffset`
        // (`:59740`) applies to every row, plus the per-row `59/60/61` nudges.
        // Row 1 is mirrored.
        let jungle_ids = world.jungle_bg();
        let jungle = front_layers(
            jungle_ids,
            [
                (1.25, 0.40, 1800.0, 1660.0),
                (1.31, 0.43, 1950.0, 1840.0),
                (1.34, 0.49, 2100.0, 2060.0),
            ],
            sky.jungle_layers,
            HACK,
        );
        let jungle: [FrontLayer; 3] = std::array::from_fn(|i| {
            let nudge = match (i, jungle_ids[i]) {
                (0, 59) => -200.0,
                (1, 60) => -175.0,
                (2, 61) => -150.0,
                _ => 0.0,
            };
            FrontLayer {
                top: jungle[i].top + nudge + layer_y_offset(jungle_ids[i]),
                flip: i == 1,
                ..jungle[i]
            }
        });
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &jungle,
            bg_alpha_front[3],
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            false,
        );

        // `DrawSurfaceBG_GoodEvilDesert` (`Main.cs:60868-60926`, dispatch
        // `:59573-59577`): front styles 5/14/13 draw the Corrupt/Crimson/
        // Hallow desert variant when its set `HasAny`, else the 26/27
        // fallback dunes. The variant arm reuses `DrawSurfaceBG_Desert`
        // verbatim; the fallback is the same two-row shape with the classic
        // magic numbers (no `SetBackgroundOffsets` call — 26/27 are not in
        // the table).
        let desert_set = world.desert_bg_set();
        // The C# walks `l` ascending (`Main.cs:59529`), so 5 fires before 13
        // before 14; only one alpha is non-zero at a time in practice, but the
        // order is kept verbatim.
        for (style, variant) in [(5usize, 1usize), (13, 3), (14, 2)] {
            let alpha = bg_alpha_front[style];
            if alpha <= 0.0 {
                continue;
            }
            let rows = desert_set[variant];
            let (ids, texs) = if tr_core::world::desert_variant_has_any(rows) {
                (
                    rows,
                    [
                        sky.desert_variants[variant - 1][0],
                        sky.desert_variants[variant - 1][1],
                        sky.desert_variants[variant - 1][2],
                    ],
                )
            } else {
                (
                    [26, 27, -1],
                    [sky.desert_fallback[0], sky.desert_fallback[1], None],
                )
            };
            let d = front_layers(
                ids,
                [
                    (1.25, 0.37, 1800.0, 1750.0),
                    (1.34, 0.49, 2100.0, 2150.0),
                    (1.34, 0.49, 2100.0, 2150.0),
                ],
                texs,
                HACK,
            );
            draw_front_trees(
                r,
                camera,
                zoom,
                screen,
                &d,
                alpha,
                magic,
                sc_adj,
                tint,
                counter,
                amb_pass.as_mut(),
                false,
            );
        }

        // `DrawSurfaceBG_Hallow` (`Main.cs:60777-60866`): the rows are gated on
        // `id > 0` (`:60779`, `:60807`, `:60836`) — no style table yields 0, so
        // the `Option` guard is equivalent; the first row's `SkyManager`
        // boundary is `1.2f / parallax` (`:60788`).
        let hallow = front_layers(
            world.hallow_bg(),
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            sky.hallow_layers,
            HACK,
        );
        let hallow = [
            FrontLayer {
                // `1.2f / this.bgParallax` runs *after* `SetBackgroundOffsets`, so
                // the divisor is the overridden parallax.
                depth: 1.2 / hallow[0].parallax,
                ..hallow[0]
            },
            hallow[1],
            hallow[2],
        ];
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &hallow,
            bg_alpha_front[6],
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            false,
        );

        // `DrawSurfaceBG_Snow` (`Main.cs:60302-60383`) on `Main.snowBG`: rows
        // gated on `id >= 0` (styles 2/3/4 are all -1 — the mountains carry
        // the biome alone), same parallax/scale/top as the forest, no layer
        // mirrored, `GetLayerYOffset` on every row (263: -700). The C# body
        // has NO trailing `DrawToDepth(1f)` (that close is Mushroom's,
        // `:60186`); the stratum waits for the `DrawRemainingDepth` flush
        // (`Main.cs:59626`).
        let snow_ids = world.snow_bg();
        let snow = front_layers(
            snow_ids,
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            sky.snow_trees,
            HACK,
        );
        let snow: [FrontLayer; 3] = std::array::from_fn(|i| FrontLayer {
            top: snow[i].top + layer_y_offset(snow_ids[i]),
            ..snow[i]
        });
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &snow,
            front_snow,
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            false,
        );

        // `DrawSurfaceBG_Crimson` (`Main.cs:60189-60300`): rows gated on
        // `id > -1`. Row 0 nudges for 105/174; row 2 shifts `175` left by 1000,
        // up by 400 and adds one tile (`:60287-60292`).
        let crimson_ids = world.crimson_bg();
        let crimson = front_layers(
            crimson_ids,
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            sky.crimson_layers,
            HACK,
        );
        let crimson: [FrontLayer; 3] = std::array::from_fn(|i| {
            let (mut top, mut x_shift, mut extra) = (crimson[i].top, 0.0, 0);
            match (i, crimson_ids[i]) {
                (0, 105) => top += 50.0,
                (0, 174) => top -= 350.0,
                (2, 175) => {
                    x_shift = -1000.0;
                    top -= 400.0;
                    extra = 1;
                }
                _ => {}
            }
            FrontLayer {
                top,
                x_shift,
                extra,
                ..crimson[i]
            }
        });
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &crimson,
            bg_alpha_front[8],
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            false,
        );

        // `DrawSurfaceBG_Mushroom` (`Main.cs:59986-60187`): the only chain that
        // recolours each row. `num5` is a fresh `Main.rand.Next(28, 42)` per row
        // (`:60002`, `:60066`, `:60128`); the port derives it from the frame
        // counter instead of consuming `Main.rand`, so the draw path stays out of
        // the sim's RNG streams (documented in `docs/ANALYSIS.md`).
        let mushroom_alpha = bg_alpha_front[9];
        let mut mushroom = front_layers(
            world.mushroom_bg(),
            [
                (1.25, 0.40, 1800.0, 1400.0),
                (1.32, 0.43, 1950.0, 1675.0),
                (1.36, 0.49, 2100.0, 1950.0),
            ],
            sky.mushroom_layers,
            HACK,
        );
        // (row tint `Vector3 * 3f`, row `num`, the `num5` divisor)
        let glow = [
            ([0.1, 0.15, 0.3], 0.5, 5000.0),
            ([0.1, 0.175, 0.3], 0.625, 5000.0),
            ([0.125, 0.2, 0.3], 0.75, 3000.0),
        ];
        for (i, (rgb, mul, divisor)) in glow.into_iter().enumerate() {
            // One `Main.rand.Next(28, 42)` per row; the port derives it from the
            // frame counter and the row index so the draw stays deterministic.
            let mut seed = (counter as u64).wrapping_mul(3).wrapping_add(i as u64);
            let rand = 28 + (next_rand(&mut seed) % 14) as i32;
            let num5 = rand as f32 * 0.001 + (270.0 - mouse_text_color as f32) / divisor;
            let rgb = [rgb[0] * 3.0, rgb[1] * 3.0, rgb[2] * 3.0];
            mushroom[i].tint = Some(mushroom_tint(tint, mushroom_alpha, mul, rgb, num5));
        }
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &mushroom,
            mushroom_alpha,
            magic,
            sc_adj,
            tint,
            counter,
            amb_pass.as_mut(),
            true,
        );

        // W4-g70: `DrawSurfaceBG_Forest` for tree sets 2-4 — the `l` loop's
        // 10/11/12 arms (`Main.cs:59539-59553`), which run AFTER mushroom
        // (`l == 9`) in the 0..15 walk, each followed by its
        // `DrawChangeOverlay(1..3)`. Same chain as set 1.
        for n in 1..4 {
            let ids = world.forest_bg(n).1;
            let rows = forest_layers(ids, sky.forest_sets[n], HACK, frame_173);
            draw_front_trees(
                r,
                camera,
                zoom,
                screen,
                &rows,
                bg_alpha_front[forest_far_slot(n)],
                magic,
                sc_adj,
                tint,
                counter,
                amb_pass.as_mut(),
                false,
            );
            draw_change_overlay(r, camera, zoom, screen, forest_flash[n]);
        }
    }

    // `Main.cs:59626` `DrawRemainingDepth`: the shallowest entities (below the
    // last front-row boundary) draw after every surface layer, before the
    // underground backdrop.
    let band_max = *amb_last;
    crate::ambience::draw_range_remaining(r, ambience, lookups, &amb_ctx, amb_last);
    // W4-c39: the party sky's remainder (the nearest balloons, below the
    // last front-row boundary) flush here too (`Main.cs:59626`).
    draw_party_band(r, f32::NEG_INFINITY, &mut party_last);
    // W4-f39: the SlimeSky's remaining strata flush with the ambience's
    // (the near slimes draw in front of the near trees — the c39 balloon
    // band's layering).
    if let Some(ss) = slime_sky {
        ss.draw_band(
            r,
            &sky.slime_sky,
            camera,
            zoom,
            screen,
            origin,
            sky_color,
            f32::NEG_INFINITY,
            band_max,
        );
    }
    // W4-c40: the SandstormSky tint quad rides the SAME flush
    // (`SkyManager.DrawDepthRange(float.MinValue, _lastDepth)`,
    // `SkyManager.cs:70-74`): its `Draw` gate (`SandstormSky.cs:51`) passes
    // for any band reaching below depth 1, so the quad paints here — after
    // every surface layer, before the underground backdrop. The tracker
    // value at the flush is the C#'s `_lastDepth` (the port's `amb_last`
    // BEFORE the ambience flush consumes it); `float.MinValue` is the C#
    // minDepth.
    let sky_flush_max = *amb_last;
    crate::ambience::draw_range_remaining(r, ambience, lookups, &amb_ctx, amb_last);
    if let Some(sky) = sandstorm_sky {
        sky.draw(
            r,
            camera,
            zoom,
            [screen.x, screen.y],
            sky_color,
            sandstorm_severity,
            f32::MIN,
            sky_flush_max,
        );
    }
    // W4-f40: the lantern sky's remainder (the shallowest lanterns, below
    // the last front-row boundary) flush here too (`Main.cs:59626`).
    draw_lantern_band(r, f32::NEG_INFINITY, &mut lantern_last);
    // `Main.DrawBackground_SurfaceTransitionBackground`,
    // `Main.DrawBackground_DirtBackground`, `DrawBackground_DrawUnderworldBlackBox`,
    // `DrawBackground_DrawRockLayer` and `DrawBackground_DrawMagmaLayer`: the textured
    // underground backdrop, anchored to `worldSurface` across the whole screen, with
    // the rock bands, the animated magma and the underworld black box below it. (A
    // per-column gradient used to hang a dark "wall" underneath every player-placed
    // row of blocks.)
    if !crate::env::no_bg() {
        draw_background(
            r,
            camera,
            zoom,
            screen,
            world,
            sky,
            light,
            cave_parallax,
            anim_time,
            hell_black_box_bottom,
            // `Main.ShouldShowInvisibleBlocksAndWalls` (`Main.cs:58023`): the port
            // has no Echo Monolith / see-invisible accessory, so the wall-parallax
            // gate rides the `TR_SHOW_INVISIBLE` A/B pin (false in normal play).
            crate::env::show_invisible(),
            metrics,
            ug_back,
            shimmer_alpha,
        );
    }
}

#[cfg(test)]
mod w4_g71_surface_variant_tests {
    use super::*;
    use crate::render::bg::{bg_offsets, front_layers, layer_y_offset};

    fn sheet() -> BgTex {
        BgTex {
            uv: [0.0, 0.0, 0.5, 0.5],
            w: 100.0,
            h: 200.0,
        }
    }

    #[test]
    fn snow_style_rows_resolve_the_worlds_set_bg_ids() {
        // `DrawSurfaceBG_Snow` (`Main.cs:60302-60383`) reads `Main.snowBG`;
        // the port resolves the same triple through `World::snow_bg`. Style
        // 7's rows are [267, 266, 268] — NOT the classic [37, 38, 39] order.
        let mut w = World::new("t".into(), 40, 40);
        w.set_surface_bg_style(3, 7);
        let ids = w.snow_bg();
        assert_eq!(ids, [267, 266, 268]);
        let rows = front_layers(
            ids,
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            [Some(sheet()); 3],
            -150.0,
        );
        // Style 7's `SetBackgroundOffsets` replaces every row's magic numbers
        // (`Main.cs:60426-60443`) and the scales/parallaxes.
        assert_eq!((rows[0].scale, rows[0].parallax), (1.25 * 2.0, 0.23));
        assert_eq!((rows[0].k, rows[0].top), (1700.0, 1300.0 - 150.0));
        assert_eq!((rows[1].k, rows[1].top), (1950.0, 1600.0 - 150.0));
        assert_eq!((rows[2].scale, rows[2].parallax), (1.34 * 2.0, 0.41));
        assert_eq!((rows[2].k, rows[2].top), (2100.0, 1850.0 - 150.0));
        // Style 6's row 0 (263) carries the -700 GetLayerYOffset on top of
        // the replaced bgTopY (`Main.cs:60319`).
        w.set_surface_bg_style(3, 6);
        let ids6 = w.snow_bg();
        let rows6 = front_layers(
            ids6,
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            [Some(sheet()); 3],
            -150.0,
        );
        let top0 = rows6[0].top + layer_y_offset(ids6[0]);
        assert_eq!(top0, 1450.0 - 150.0 - 700.0);
        // Styles 2/3/4: all rows -1, nothing draws (the mountains carry it).
        for style in [2, 3, 4] {
            w.set_surface_bg_style(3, style);
            assert_eq!(w.snow_bg(), [-1, -1, -1], "snow style {style}");
            assert_eq!(w.snow_mountains()[0], 98);
        }
        // Style 8: two rows, third -1; the mountains stay 35/36.
        w.set_surface_bg_style(3, 8);
        assert_eq!(w.snow_bg(), [299, 298, -1]);
        assert_eq!(w.snow_mountains(), [35, 36]);
    }

    #[test]
    fn desert_pure_rows_follow_the_style_including_the_oasis_third_row() {
        // `DrawSurfaceBG_Desert` (`Main.cs:61019-61096`) on Pure.Backgrounds.
        let mut w = World::new("t".into(), 40, 40);
        // Style 0: [21, 20, -1], no offsets — the caller's rows win.
        let ids = w.desert_bg_set()[0];
        assert_eq!(ids, [21, 20, -1]);
        let rows = front_layers(
            ids,
            [
                (1.25, 0.37, 1800.0, 1750.0),
                (1.34, 0.49, 2100.0, 2150.0),
                (1.34, 0.49, 2100.0, 2150.0),
            ],
            [Some(sheet()), Some(sheet()), None],
            -150.0,
        );
        assert_eq!((rows[0].scale, rows[0].parallax), (2.5, 0.37));
        assert_eq!((rows[0].k, rows[0].top), (1800.0, 1600.0));
        assert!(rows[2].tex.is_none());
        // Style 4 (oasis): [248, 249, 250], all three rows replaced
        // (`Main.cs:60444-60461`); the third row draws at scale 1.7*2.
        w.set_surface_bg_style(6, 4);
        let ids4 = w.desert_bg_set()[0];
        assert_eq!(ids4, [248, 249, 250]);
        let rows4 = front_layers(
            ids4,
            [
                (1.25, 0.37, 1800.0, 1750.0),
                (1.34, 0.49, 2100.0, 2150.0),
                (1.34, 0.49, 2100.0, 2150.0),
            ],
            [Some(sheet()); 3],
            -150.0,
        );
        assert_eq!((rows4[0].k, rows4[0].top), (1800.0, 1100.0 - 150.0));
        assert_eq!((rows4[1].k, rows4[1].top), (1950.0, 1200.0 - 150.0));
        assert_eq!((rows4[2].scale, rows4[2].parallax), (1.70 * 2.0, 0.49));
        assert_eq!((rows4[2].k, rows4[2].top), (2000.0, 1000.0 - 150.0));
        // Style 2: 207's parallax-only override (0.32), tops kept.
        w.set_surface_bg_style(6, 2);
        let ids2 = w.desert_bg_set()[0];
        assert_eq!(ids2, [207, 208, -1]);
        assert_eq!(bg_offsets(207), Some((None, Some(0.32), 0, 0)));
        // Style 52: the four variants' middle rows differ, far rows shared.
        w.set_surface_bg_style(6, 52);
        let set = w.desert_bg_set();
        assert_eq!(set[0], [306, 304, -1]);
        assert_eq!(set[1], [310, 308, -1]); // Corrupt
        assert_eq!(set[2], [318, 316, -1]); // Crimson
        assert_eq!(set[3], [314, 312, -1]); // Hallow
        // The 51-family middle rows share the (2400, 1900, 0.35) arm.
        let m = front_layers(
            set[1],
            [
                (1.25, 0.37, 1800.0, 1750.0),
                (1.34, 0.49, 2100.0, 2150.0),
                (1.34, 0.49, 2100.0, 2150.0),
            ],
            [Some(sheet()); 3],
            -150.0,
        );
        assert_eq!((m[0].k, m[0].top, m[0].parallax), (1800.0, 1550.0, 0.26));
        assert_eq!((m[1].k, m[1].top, m[1].parallax), (2400.0, 1750.0, 0.35));
    }

    #[test]
    fn good_evil_desert_picks_the_variant_or_the_26_27_fallback() {
        // `DrawSurfaceBG_GoodEvilDesert` (`Main.cs:60868-60884`): style 5 ->
        // Corrupt, 14 -> Crimson, 13 -> Hallow, each gated on HasAny; the
        // fallback is 26/27 with the classic magic numbers.
        let mut w = World::new("t".into(), 40, 40);
        // Style 0: only Pure has rows — every variant falls back.
        let set = w.desert_bg_set();
        for v in 1..4 {
            assert!(!tr_core::world::desert_variant_has_any(set[v]));
        }
        // Style 51: all four sets populated.
        w.set_surface_bg_style(6, 51);
        let set = w.desert_bg_set();
        assert_eq!(set[1], [310, 307, -1]);
        assert_eq!(set[2], [318, 315, -1]);
        assert_eq!(set[3], [314, 311, -1]);
        for v in 1..4 {
            assert!(tr_core::world::desert_variant_has_any(set[v]));
        }
        // The fallback rows keep the caller's scale/parallax/tops (26/27 are
        // not in SetBackgroundOffsets).
        let f = front_layers(
            [26, 27, -1],
            [
                (1.25, 0.37, 1800.0, 1750.0),
                (1.34, 0.49, 2100.0, 2150.0),
                (1.34, 0.49, 2100.0, 2150.0),
            ],
            [Some(sheet()); 3],
            -150.0,
        );
        assert_eq!((f[0].scale, f[0].parallax), (2.5, 0.37));
        assert_eq!((f[0].k, f[0].top), (1800.0, 1600.0));
        assert_eq!((f[1].k, f[1].top), (2100.0, 2000.0));
    }

    #[test]
    fn ocean_style_resolves_the_single_far_layer_id() {
        // `DrawSurfaceBG_BackMountainsStep2` draws `Main.oceanBG` at far slot
        // 3 (`Main.cs:59916-59920`); the id comes from `setBG(7)`.
        let mut w = World::new("t".into(), 40, 40);
        assert_eq!(w.ocean_bg(), 28);
        w.set_surface_bg_style(7, 5);
        assert_eq!(w.ocean_bg(), 283);
        // The shimmer ocean sits 800 px higher (`GetLayerYOffset` 283).
        assert_eq!(layer_y_offset(w.ocean_bg()), -800.0);
        w.set_surface_bg_style(7, 6);
        assert_eq!(w.ocean_bg(), 332);
        assert_eq!(layer_y_offset(332), -25.0);
        w.set_surface_bg_style(7, 7);
        assert_eq!(w.ocean_bg(), 340);
        assert_eq!(layer_y_offset(340), -25.0);
    }

    #[test]
    fn jungle_style_six_rows_are_already_ported_verbatim() {
        // The jungle variant lane (g70's neighbour) is landed: style 6 =
        // [302, 301, 300] with the 2.25/2.36/2.41 scales and the reversed
        // row order; this witness pins the seam the g71 tables must not
        // disturb (`WorldGen.cs:7310-7314`, `Main.cs:60631-60648`).
        let mut w = World::new("t".into(), 40, 40);
        w.set_surface_bg_style(2, 6);
        let ids = w.jungle_bg();
        assert_eq!(ids, [302, 301, 300]);
        let rows = front_layers(
            ids,
            [
                (1.25, 0.40, 1800.0, 1660.0),
                (1.31, 0.43, 1950.0, 1840.0),
                (1.34, 0.49, 2100.0, 2060.0),
            ],
            [Some(sheet()); 3],
            -150.0,
        );
        assert_eq!(rows[0].scale, 2.25 * 2.0);
        assert_eq!((rows[0].k, rows[0].top), (1400.0, 950.0 - 150.0));
        assert_eq!(rows[1].scale, 2.36 * 2.0);
        assert_eq!((rows[2].k, rows[2].top), (3000.0, 2050.0 - 150.0));
        // Row 0 (302) is a 2x2 animated sheet at 15 ticks.
        assert_eq!(rows[0].frame_step, 15);
    }
}

#[cfg(test)]
mod cloud_star_tests {
    use super::*;

    #[test]
    fn cloud_parallax_matches_the_three_scale_buckets() {
        // `Cloud.GetParallax` (`Cloud.cs:318-338`): 0.07/0.19/0.23 over the
        // squared factor; the nudged bucket edges (`Cloud.cs:383-390`) land in
        // the low buckets.
        let small = crate::cloud::cloud_parallax(0.9999);
        let mid = crate::cloud::cloud_parallax(1.1499);
        let big = crate::cloud::cloud_parallax(1.3);
        assert!((small - 0.07 * (1.074_95 * 1.074_95)).abs() < 1e-5);
        assert!((mid - 0.19 * (1.074_9 * 1.074_9)).abs() < 1e-5);
        assert!((big - 0.23 * (1.075 * 1.075)).abs() < 1e-5);
        assert!(small < mid && mid < big);
    }

    #[test]
    fn night_light_floor_follows_the_moon_phase() {
        // `Main.cs:63792-63829` via the byte-space chain (`tr_core::sky_color`):
        // midnight (t = 16200) with the ramp floor of 5 lifted to the phase
        // minimum — 19 at the full moon, 11 at the new one.
        let mut i = tr_core::sky_color::BackColorInputs {
            time: 16200.0,
            day: false,
            moon_phase: 0,
            blood_moon: false,
            eclipse: false,
            eclipse_light: 0.0,
            cloud_alpha: 0.0,
            atmo: 1.0,
            graveyard: 0.0,
            corruption: 0.0,
            crimson: 0.0,
            jungle: 0.0,
            mushroom: 0.0,
            lightning: 0.0,
            remix: false,
            dont_starve: false,
            not_retro: true,
        };
        assert_eq!(tr_core::sky_color::set_back_color(&i).bg, [19, 19, 19]);
        i.moon_phase = 4;
        assert_eq!(tr_core::sky_color::set_back_color(&i).bg, [11, 11, 11]);
        // The right-half mirror: phase 5 quarter floors at 13.
        i.moon_phase = 5;
        assert_eq!(tr_core::sky_color::set_back_color(&i).bg, [13, 13, 13]);
        // Daylight ignores the floor.
        i.day = true;
        i.time = 13500.0;
        assert!(tr_core::sky_color::set_back_color(&i).bg[0] > 23);
        // `Main.cs:63792-63829`: midnight (t = 16200) with the ramp floor of 5
        // lifted to the phase minimum — 19 at the full moon, 11 at the new one.
        let full = color_of_the_skies(16200.0, false, 0, false);
        let empty = color_of_the_skies(16200.0, false, 4, false);
        let full_byte = full[0] * 255.0;
        let empty_byte = empty[0] * 255.0;
        assert!((full_byte - 19.0).abs() < 1e-4, "full moon {full:?}");
        assert!((empty_byte - 11.0).abs() < 1e-4, "new moon {empty:?}");
        // The right-half mirror: phase 5 quarter floors at 13.
        let quarter = color_of_the_skies(16200.0, false, 5, false)[0] * 255.0;
        assert!((quarter - 13.0).abs() < 1e-4, "quarter {quarter}");
        // Daylight ignores the floor.
        let day = color_of_the_skies(13500.0, true, 4, false);
        assert!(day[0] > 0.09, "dawn stays dawn: {day:?}");
    }

    #[test]
    fn horizon_phase_shifts_the_morning_half() {
        // `Main.cs:2792-2805`: before mid-day the phase belongs to the
        // previous moon phase (mod 8).
        assert_eq!(horizon_phase(3, true, 20000.0), 2);
        assert_eq!(horizon_phase(3, true, 30000.0), 3);
        assert_eq!(horizon_phase(0, true, 10000.0), 7);
        assert_eq!(horizon_phase(0, false, 10000.0), 0);
    }

    #[test]
    fn horizon_visibilities_follow_the_day_windows() {
        // Night: both closed (`NextHorizonRenderer.cs:225-229`).
        let (sunset, sunrise) = horizon_visibilities(10000.0, false, 1.0, 0.0, 0.0, false);
        assert_eq!((sunset, sunrise), (0.0, 0.0));
        // Noon: the dawn window (0..10800) has closed, the dusk one (43200+)
        // has not opened.
        let (sunset, sunrise) = horizon_visibilities(27000.0, true, 1.0, 0.0, 0.0, false);
        assert_eq!((sunset, sunrise), (0.0, 0.0));
        // Dawn peak (t=1350): the sunrise at half strength, the sunset closed.
        let (sunset, sunrise) = horizon_visibilities(1350.0, true, 1.0, 0.0, 0.0, false);
        assert!(sunset < 1e-6, "sunset={sunset}");
        assert!((sunrise - 0.5).abs() < 1e-6, "sunrise={sunrise}");
        // Dusk peak (t=50400): the sunset at full strength.
        let (sunset, sunrise) = horizon_visibilities(50400.0, true, 1.0, 0.0, 0.0, false);
        assert!((sunset - 1.0).abs() < 1e-6, "sunset={sunset}");
        assert!(sunrise < 1e-6, "sunrise={sunrise}");
        // Cloud cover squares into the factor.
        let (sunset_clear, _) = horizon_visibilities(50400.0, true, 1.0, 0.0, 0.0, false);
        let (sunset_cloudy, _) = horizon_visibilities(50400.0, true, 1.0, 0.5, 0.0, false);
        assert!((sunset_cloudy - sunset_clear * 0.25).abs() < 1e-6);
        // The mushroom factor (`:203`) scales linearly; the eclipse
        // (`:219-223`) kills both.
        let (sunset_shroom, _) = horizon_visibilities(50400.0, true, 1.0, 0.0, 0.5, false);
        assert!((sunset_shroom - sunset_clear * 0.5).abs() < 1e-6);
        assert_eq!(
            horizon_visibilities(50400.0, true, 1.0, 0.0, 0.0, true),
            (0.0, 0.0)
        );
    }

    #[test]
    fn modify_horizon_light_lifts_dark_dawn_toward_the_gradient() {
        // A dawn-dark sky (25,35,35) with the sunrise at half strength lifts
        // toward Sunrise_Blue[7] (68,144,149) — the max-channel byte blend
        // (`NextHorizonRenderer.cs:183-194`): R = 25 + (68-25)*0.5 = 46.5 → 46,
        // G = 35 + 54.5 = 89.5 → 89, B = 35 + 57 = 92.
        let mut c = [25u8, 35, 35];
        modify_horizon_light(&mut c, 1, true, 1350.0, 1.0, 0.0, 0.0, false);
        assert_eq!(c, [46, 89, 92]);
        // The white noon colour cannot rise (the max blend + zero visibility).
        let mut noon = [255u8, 255, 255];
        modify_horizon_light(&mut noon, 1, true, 27000.0, 1.0, 0.0, 0.0, false);
        assert_eq!(noon, [255, 255, 255]);
    }

    #[test]
    fn atmosphere_factor_clamps_to_the_unit_range() {
        // Deep underground saturates at 1, high sky bottoms out at 0.
        assert_eq!(atmosphere_factor(3000.0, 300.0, 4200), 1.0);
        assert_eq!(atmosphere_factor(0.0, 300.0, 4200), 0.0);
    }

    #[test]
    fn celestial_power_ramps_at_the_half_day_edges() {
        // `NextHorizonRenderer.cs:311-321`: in over 0..3600, out over the
        // last 1800 ticks. Noon/midnight sit at full strength.
        assert_eq!(celestial_effect_power(27000.0, true), 1.0);
        assert_eq!(celestial_effect_power(16200.0, false), 1.0);
        assert_eq!(celestial_effect_power(0.0, true), 0.0);
        assert!((celestial_effect_power(1800.0, true) - 0.5).abs() < 1e-6);
        // The day's out-ramp: 53100 is the midpoint of 52200..54000.
        assert!((celestial_effect_power(53100.0, true) - 0.5).abs() < 1e-6);
        assert_eq!(celestial_effect_power(54000.0, true), 0.0);
        // The night mirrors it on 30600..32400.
        assert!((celestial_effect_power(31500.0, false) - 0.5).abs() < 1e-6);
        assert_eq!(celestial_effect_power(32400.0, false), 0.0);
    }

    #[test]
    fn event_moon_sheets_share_the_phase_row_layout() {
        // `Main.cs:62970-62978`: the event moons draw
        // `Rectangle(0, width * moonPhase, width, width)` — the same 8-row
        // slice as Moon_0..8. A synthetic 50x400 sheet exercises the row
        // math the draw block uses (phase 3 → v spans rows 3..4 of 8).
        let m = BgTex { uv: [0.0, 0.0, 0.1, 0.8], w: 50.0, h: 400.0 };
        let dv = (m.uv[3] - m.uv[1]) / 8.0;
        let v0 = m.uv[1] + dv * 3.0;
        assert!((dv - 0.1).abs() < 1e-6);
        assert!((v0 - 0.3).abs() < 1e-6);
    }

    #[test]
    fn stars_follow_the_spawn_recipe() {
        // `Star.SpawnStars` (`Star.cs:89`): 200..400 stars, reproducible per
        // seed — the live pool (`crate::star_field`, W4-g70 lane 8).
        let a = crate::star_field::StarField::new(0xDEAD_BEEF);
        let b = crate::star_field::StarField::new(0xDEAD_BEEF);
        assert_eq!(a.stars.len(), b.stars.len());
        assert!((200..400).contains(&a.stars.len()));
        for (sa, sb) in a.stars.iter().zip(b.stars.iter()) {
            assert_eq!(sa.x, sb.x);
            assert_eq!(sa.tex, sb.tex);
            assert!(sa.scale > 0.0);
        }
    }

    #[test]
    fn cloud_pass_cy_formulas_are_verbatim() {
        // `DrawClouds_Distant` (`Main.cs:59699`): UNSCALED Y + the
        // magic*750+830+scAdj head (hack 0). At 720p the closer/closest
        // passes scale Y by 720/600 = 1.2 (`:59678`, `:59659`).
        assert_eq!(cloud_distant_cy(-90.0, 0.5, 20.0), -90.0 + 1225.0);
        // Closer: `Y * (h/600) + (bgTopY - 50) + 200` — the +150 net.
        let closer = cloud_closer_cy(-100.0, 720.0, 1000.0);
        assert!((closer - (-120.0 + 1150.0)).abs() < 1e-2, "closer {closer}");
        // Closest: `Y * (h/600) - 100 + bgTopY * 1.01 - 150`.
        let closest = cloud_closest_cy(-100.0, 720.0, 1000.0);
        let want = -120.0 - 100.0 + (1000.0 * 1.01 - 150.0);
        assert!((closest - want).abs() < 1e-2, "closest {closest} vs {want}");
        // At the 600 px reference screen the scale term is identity.
        assert_eq!(cloud_closer_cy(-100.0, 600.0, 1000.0), -100.0 + 1150.0);
    }

    #[test]
    fn draw_cloud_color_replays_the_byte_chain() {
        // `NextHorizonRenderer.DrawCloud` (`:241-252`): cloudColor scales the
        // sky bytes by min(scale*Alpha, 1) with the byte truncation, pass 1
        // pulls R/G, atmo < 1 dims, then globalCloudAlpha multiplies.
        let white = [1.0, 1.0, 1.0];
        // scale 0.8, Alpha 1: every channel (byte)(255 * 0.8) = 204; the
        // distant pull: R = (byte)(204 * 0.64) = 130, G = (byte)(204 * 0.81)
        // = 165 (0.9 * (0.8+1)/2 = 0.81).
        let c = draw_cloud_color(white, 0.8, 1.0, 1, 1.0, 1.0);
        assert_eq!(c[0], 130.0 / 255.0);
        assert_eq!(c[1], 165.0 / 255.0);
        assert_eq!(c[2], 204.0 / 255.0);
        assert_eq!(c[3], 204.0 / 255.0);
        // Pass 2 leaves R/G at the cloudColor value.
        let c2 = draw_cloud_color(white, 0.8, 1.0, 2, 1.0, 1.0);
        assert_eq!(c2, [204.0 / 255.0; 4]);
        // Alpha 0.5 halves the factor: (byte)(255 * 0.4) = 102.
        let faded = draw_cloud_color(white, 0.8, 0.5, 2, 1.0, 1.0);
        assert_eq!(faded, [102.0 / 255.0; 4]);
        // atmo 0.5 truncates (byte)(204 * 0.5) = 102, then globalCloudAlpha
        // 0.5: (byte)(102 * 0.5) = 51.
        let dim = draw_cloud_color(white, 0.8, 1.0, 2, 0.5, 0.5);
        assert_eq!(dim, [51.0 / 255.0; 4]);
        // globalCloudAlpha 0 (atmo 0 deep underground) blacks the sprite out.
        let gone = draw_cloud_color(white, 1.0, 1.0, 3, 0.0, 0.0);
        assert_eq!(gone, [0.0; 4]);
    }

    /// W4-g69 doc-sweep — the rain-cloud darkening census: `Cloud.cloudColor`
    /// (`Cloud.cs:227-235`) carries NO per-cloud rain tint; the darkening is
    /// (a) the dark storm TYPES 18-21 spawned while raining/overcast
    /// (`Cloud.cs:116-127`, the `rain_spawn_takes_the_dark_type_branch`
    /// witness in `cloud.rs`) and (b) the GLOBAL `SetBackColor` dim
    /// `1 - cloudAlpha * 0.9 * atmo` (`Main.cs:63546-63552`), byte-truncated,
    /// gated on `cloudAlpha > 0 && !remixWorld` — no 0.1 floor.
    #[test]
    fn w4_g69_rain_dim_sky_is_the_global_byte_truncated_formula() {
        let white = [1.0, 1.0, 1.0];
        // cloudAlpha 1, atmo 1: (byte)(255 * 0.1) = 25.
        assert_eq!(rain_dim_sky(white, 1.0, 1.0, false), [25.0 / 255.0; 3]);
        // cloudAlpha 0.5: num3 = 0.55, (byte)(255 * 0.55) = 140 (140.25).
        assert_eq!(rain_dim_sky(white, 0.5, 1.0, false), [140.0 / 255.0; 3]);
        // atmo 0 (deep underground): num3 = 1 — no dim at all.
        assert_eq!(rain_dim_sky(white, 1.0, 0.0, false), white);
        // cloudAlpha 0: the gate skips the multiply entirely.
        assert_eq!(rain_dim_sky(white, 0.0, 1.0, false), white);
        // remixWorld: the C# never dims (`:63546`'s `!Main.remixWorld`).
        assert_eq!(rain_dim_sky(white, 1.0, 1.0, true), white);
        // The RAW alpha can exceed 1 (the port's TR_RAIN pin): num3 goes
        // NEGATIVE and the unchecked byte cast wraps through the low byte
        // (the C# `(byte)` in unchecked context, `as i32 as u8`): num3 =
        // 1 - 1.5*0.9 = -0.35, (int)(255 * -0.35) = -89 -> (byte)167.
        // Vanilla maxRaining ≤ 1 keeps num3 ≥ 0.1, so only pins reach it.
        assert_eq!(rain_dim_sky(white, 1.5, 1.0, false), [167.0 / 255.0; 3]);
        // A non-white channel truncates independently: (byte)(200 * 0.55)
        // = 110, (byte)(100 * 0.55) = 55, (byte)(50 * 0.55) = 27.
        let tinted = [200.0 / 255.0, 100.0 / 255.0, 50.0 / 255.0];
        assert_eq!(
            rain_dim_sky(tinted, 0.5, 1.0, false),
            [110.0 / 255.0, 55.0 / 255.0, 27.0 / 255.0]
        );
    }

    /// W4-g69 retire sweep (g69d-star-visual) — `DrawStarsInBackground`'s
    /// heads + gate (`Main.cs:63030-63058`): the RAW `cloudAlpha * atmo`
    /// (NOT the storm deck's cloudBGAlpha the stand-in read), the remix
    /// `||` arm, the graveyard opacity kill and the shimmer sky dim + the
    /// `shimmerAlpha == 1` pass skip (`:62086-62088`).
    #[test]
    fn w4_g69_the_star_gate_reads_the_raw_cloud_alpha() {
        let night = [100.0 / 255.0; 3];
        // Clear sky: 255 - 100 - 25 = 130 > 0 — open, opacity 1, sky unchanged.
        let (op, sky, open) = star_pass_heads(night, 0.0, 0.0, 0.0, 1.0, false);
        assert_eq!(op, 1.0);
        assert_eq!(sky, night);
        assert!(open);
        // cloudAlpha 0.5: 255*0.5 - 100 - 25 = 2.5 > 0 — still open.
        assert!(star_pass_heads(night, 0.0, 0.0, 0.5, 1.0, false).2);
        // cloudAlpha 0.6: 255*0.4 - 100 - 25 = -23 <= 0 — CLOSED, even though
        // the storm deck (cloudBGAlpha) is empty: the old stand-in read the
        // storm and kept the stars visible through a heavy rain.
        assert!(!star_pass_heads(night, 0.0, 0.0, 0.6, 1.0, false).2);
        // atmo 0 (deep underground): the cloud term vanishes — open again.
        assert!(star_pass_heads(night, 0.0, 0.0, 1.0, 0.0, false).2);
        // remixWorld: the `||` arm draws the stars UNCONDITIONALLY.
        assert!(star_pass_heads(night, 0.0, 0.0, 1.0, 1.0, true).2);
        // Graveyard intensity 1: num2 = 1 - 1.4 <= 0 — the whole field dies
        // (`:63040-63043`), remix or not.
        let (op, _, open) = star_pass_heads(night, 1.0, 0.0, 0.0, 1.0, true);
        assert_eq!(op, 0.0);
        assert!(!open);
        // Half graveyard: the opacity scales 1 - 0.5*1.4 = 0.3.
        let (op, _, open) = star_pass_heads(night, 0.5, 0.0, 0.0, 1.0, false);
        assert!((op - 0.3).abs() < 1e-6, "opacity {op}");
        assert!(open);
        // Shimmer 0.5 dims the sky bytes: (byte)(255 * 0.5) = 127 — the gate
        // AND DrawStar read the dimmed colour (`:63046-63050`).
        let white = [1.0; 3];
        let (_, sky, open) = star_pass_heads(white, 0.0, 0.5, 0.0, 1.0, false);
        assert_eq!(sky, [127.0 / 255.0; 3]);
        assert!(open);
        // shimmerAlpha 1: the pass is skipped wholesale (`:62086-62088`).
        assert!(!star_pass_heads(white, 0.0, 1.0, 0.0, 1.0, true).2);
    }

    /// W4-g69 retire sweep (g69d-star-visual) — `DrawStar`'s colour chain
    /// (`Main.cs:63066-63110`): the per-channel `(int)` truncation BEFORE the
    /// /3 average, the `(double) * 1.4` cap-255 step, the `starOpacity`
    /// graveyard multiply, and the shimmer i%3 `twinkle²` channel tint +
    /// `Color.Lerp` + `*= 1 - shimmer` (the artificial-false arm).
    #[test]
    fn w4_g69_draw_star_color_replays_the_byte_chain() {
        let black = [0.0; 3];
        // Black night sky, twinkle 1: (255-0-100)*1 = 155 per channel,
        // avg 155, (int)(155.0 * 1.4) = 216 (the f64 1.4 is
        // 1.39999999999999991…, so 216.99999… truncates to 216 — the C#
        // `(int)((double)num3 * 1.4)` lands the same byte).
        assert_eq!(
            draw_star_color(black, 1.0, 1.0, 0.0, 0),
            Some([216; 3])
        );
        // twinkle 0.6 (the f32 0.6 is 0.60000002384…): 155*0.6 =
        // 93.0000036 -> (int) 93, (int)(93*1.4) = 130 (130.19999… truncates).
        assert_eq!(
            draw_star_color(black, 0.6, 1.0, 0.0, 0),
            Some([130; 3])
        );
        // Bright sky kills the star: (255-200-100) < 0 -> n3 <= 0 -> None
        // (`:63072-63074`).
        assert_eq!(draw_star_color([200.0 / 255.0; 3], 1.0, 1.0, 0.0, 0), None);
        // Graveyard opacity 0.65 (intensity 0.25): (byte)(216 * 0.65) = 140.
        assert_eq!(
            draw_star_color(black, 1.0, 0.65, 0.0, 0),
            Some([140; 3])
        );
        // Shimmer 0.5, twinkle 0.8, i%3 == 1 tints B: base (int)(124*1.4) =
        // 173; the B channel takes twinkle² = 0.64: (byte)(173*0.64) = 110;
        // Lerp 0.5: (int)(173 + (110-173)*0.5) = 141; *= 0.5:
        // R/G (byte)(173*0.5) = 86, B (byte)(141*0.5) = 70.
        assert_eq!(
            draw_star_color(black, 0.8, 1.0, 0.5, 1),
            Some([86, 86, 70])
        );
        // The i%3 channel map (`:63090-63100`): 0 -> G, 1 -> B, 2 -> R.
        assert_eq!(
            draw_star_color(black, 0.8, 1.0, 0.5, 0),
            Some([86, 70, 86])
        );
        assert_eq!(
            draw_star_color(black, 0.8, 1.0, 0.5, 2),
            Some([70, 86, 86])
        );
    }

    #[test]
    fn live_pool_feeds_the_passes_after_reset() {
        // W4-g69: the draw consumes the live 200-slot pool — after the
        // `DrawBG` reset latch (`Main.cs:63980-63984`) exactly `numClouds`
        // slots are active with Alpha 1 (0x0 sheets: no overlap rejection,
        // the `cloud.rs` reset-witness convention), and the clamps hold.
        let mut field = crate::cloud::CloudField::new(7);
        let mut w = World::new("t".into(), 4200, 1200);
        w.seed = 7;
        let env = crate::cloud::CloudEnv::gather(
            &w,
            Vec2::new(1280.0, 720.0),
            0.3,
            0.0,
            false,
            1,
            0.0,
            0.0,
        );
        field.num_clouds = 100;
        field.reset_clouds(&env, &assets::Lookups::default());
        field.reset_pending = false;
        assert_eq!(field.slots.iter().filter(|c| c.active).count(), 100);
        for c in field.slots.iter().filter(|c| c.active) {
            assert!((0.7..=1.3).contains(&c.scale));
            assert!(c.rotation.abs() <= 0.1 + 1e-6);
            assert_eq!(c.alpha, 1.0);
        }
    }

    #[test]
    fn day_cycle_sky_color_lerps_per_csharp() {
        // `Main.cs:63452-63496` (`SetBackColor`): the day is pure white from
        // 13500 to 37800 (noon is `Color.White`), the 37800..45900 sunset ramp
        // pulls it to (235,120,170) and past 45900 to the (35,35,35) dusk
        // floor. The 0..13500 sunrise ramp runs 25..255 on the red channel.
        // W4-g70 merger re-pin: the f32 `color_of_the_skies` approximation
        // retired onto `tr_core::sky_color::set_back_color`'s byte space —
        // the same expectations read off `BackColors.bg` (the byte cast
        // truncates, so the ramp values are exact integers).
        let at = |time: f64| {
            let mut i = tr_core::sky_color::BackColorInputs {
                time,
                day: true,
                moon_phase: 0,
                blood_moon: false,
                eclipse: false,
                eclipse_light: 0.0,
                cloud_alpha: 0.0,
                atmo: 1.0,
                graveyard: 0.0,
                corruption: 0.0,
                crimson: 0.0,
                jungle: 0.0,
                mushroom: 0.0,
                lightning: 0.0,
                remix: false,
                dont_starve: false,
                not_retro: true,
            };
            i.time = time;
            tr_core::sky_color::set_back_color(&i).bg
        };
        let noon = at(27000.0);
        assert_eq!(noon, [255, 255, 255], "noon is pure white");
        let morning = at(6750.0);
        assert_eq!(morning[0], 140, "sunrise red");
        assert_eq!(morning[1], 145, "sunrise green");
        let sunset = at(45900.0);
        assert_eq!(sunset[0], 235, "sunset red");
        assert_eq!(sunset[1], 120, "sunset green");
        assert_eq!(sunset[2], 170, "sunset blue");
        let dusk = at(53999.0);
        assert!(dusk[0] <= 36, "the dusk floor: {dusk:?}");
        let noon = color_of_the_skies(27000.0, true, 0, false);
        assert_eq!(noon, [1.0, 1.0, 1.0], "noon is pure white");
        let morning = color_of_the_skies(6750.0, true, 0, false);
        let byte = |v: f32| v * 255.0;
        assert!((byte(morning[0]) - 140.0).abs() < 1e-3, "sunrise red");
        assert!((byte(morning[1]) - 145.0).abs() < 1e-3, "sunrise green");
        let sunset = color_of_the_skies(45900.0, true, 0, false);
        assert!((byte(sunset[0]) - 235.0).abs() < 1e-3, "sunset red");
        assert!((byte(sunset[1]) - 120.0).abs() < 1e-3, "sunset green");
        assert!((byte(sunset[2]) - 170.0).abs() < 1e-3, "sunset blue");
        let dusk = color_of_the_skies(53999.0, true, 0, false);
        assert!((byte(dusk[0]) - 35.0).abs() < 0.1, "the dusk floor: {dusk:?}");
        assert!(dusk[0] < sunset[0] && sunset[0] < noon[0], "the day darkens");
    }

    /// W4-g12 doc-sweep: the blood-moon night sky (`Main.cs:63495-63519` +
    /// the `:63830-63844` floor of 25). The red sky replaces both forest
    /// ramps; the 25 floor is a byte compare AFTER the moon-phase floor.
    #[test]
    fn w4_g12_blood_moon_sky_and_the_25_floor() {
        // Midnight: the raw ramp is (75, 15, 15); the moon-phase floor
        // (full moon 19) leaves R alone, then the blood floor lifts G/B
        // to 25 (`:63836-63843`).
        let mid = color_of_the_skies(16200.0, false, 0, true);
        let b = |v: f32| (v * 255.0).round() as i32;
        assert_eq!([b(mid[0]), b(mid[1]), b(mid[2])], [75, 25, 25]);
        // New moon: the phase floor 11 never beats the blood floor 25.
        let mid_new = color_of_the_skies(16200.0, false, 4, true);
        assert_eq!(mid_new, mid, "the 25 floor dominates every phase floor");
        // Early night t=1: the ramp is (35, 35, 35) — all above 25.
        let early = color_of_the_skies(0.0, false, 0, true);
        assert_eq!([b(early[0]), b(early[1]), b(early[2])], [35, 35, 35]);
        // Daylight ignores the blood arm entirely (the C# gates on
        // `!Main.dayTime`, `:63493`).
        let noon = color_of_the_skies(27000.0, true, 0, true);
        assert_eq!(noon, [1.0, 1.0, 1.0]);
        // Without the blood moon the forest ramps are untouched.
        let forest = color_of_the_skies(16200.0, false, 4, false);
        assert!((forest[0] * 255.0 - 11.0).abs() < 1e-4);
    }
}
