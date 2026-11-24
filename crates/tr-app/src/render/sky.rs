//! The sky pass: `Main.SetBackColor`'s sky colours, the deterministic cloud/star
//! assets and `DrawSunAndMoon`/`DrawSurfaceBG` (`draw_sky`). The textured
//! underground backdrop it tails into lives in [`super::bg`].
use super::BgTex;
use super::bg::{
    FrontLayer, bg_scale_adjust, draw_background, draw_bg_layer, draw_front_trees, front_layers,
    layer_y_offset, mushroom_tint, push_screen_rot,
};
use crate::app::next_rand;
use crate::assets;
use glam::Vec2;
use tr_core::lighting::LightMap;
use tr_core::{TILE_PX, World};
use tr_render::{Atlas, SpriteRenderer};

fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        1.0,
    ]
}

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

/// `Main.SetBackColor`: the colour the sky and surface background are tinted with
/// (`Main.ColorOfTheSkies`), as a function of world time.
///
/// Day runs `0..54000` (0 = sunrise, 27000 = noon), night runs `0..32400`
/// (16200 = midnight). Between 13500 and 37800 the colour is pure white, which is why
/// vanilla daytime tiles are lit at 100 %.
pub(crate) fn color_of_the_skies(time: f64, day: bool, moon_phase: u8) -> [f32; 3] {
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
        // blood-moon floor of 25 (`:63830-63844`) lands with the blood-moon
        // system (M4 events); its night ramp is not ported either.
        let (r, g, b) = if time < 16200.0 {
            let t = 1.0 - time / 16200.0;
            (t * 30.0 + 5.0, t * 30.0 + 5.0, t * 30.0 + 5.0)
        } else {
            let t = (time / 32400.0 - 0.5) * 2.0;
            (t * 20.0 + 5.0, t * 30.0 + 5.0, t * 30.0 + 5.0)
        };
        // `Enums/MoonPhase.cs`: Full=0, Empty=4, the right-half mirror behind.
        const FLOORS: [f64; 8] = [19.0, 17.0, 15.0, 13.0, 11.0, 13.0, 15.0, 17.0];
        let floor = (FLOORS[(moon_phase & 7) as usize] / 255.0) as f32;
        [
            byte(r).max(floor),
            byte(g).max(floor),
            byte(b).max(floor),
        ]
    }
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
    let n = max_tiles_x as f32 / 4200.0;
    let n = n * n;
    let mut atmo = (y / 16.0 - (65.0 + 10.0 * n)) / (world_surface as f32 / 5.0);
    atmo.clamp(0.0, 1.0)
}

/// `Utils.Remap`: the clamped from→to lerp the visibility chain uses.
fn remap(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
    let t = ((value - from_min) / (from_max - from_min)).clamp(0.0, 1.0);
    to_min + (to_max - to_min) * t
}

/// `NextHorizonRenderer.GetVisibilities` (`NextHorizonRenderer.cs:196-234`):
/// the sunrise/sunset light-tint strengths. `mushroom_influence` is
/// `Main.SmoothedMushroomLightInfluence` (0 until the mushroom biome lands)
/// and `eclipse` is false until M4 events — both terms omitted here.
pub(crate) fn horizon_visibilities(time: f64, day: bool, atmo: f32, cloud_alpha: f32) -> (f64, f64) {
    let mut sunrise = 1.0f64;
    let mut sunset = 1.0f64;
    let num = 1.0 * atmo as f64;
    let num2 = 1.0 - cloud_alpha as f64;
    let num = num * num2 * num2;
    sunrise *= num;
    sunset *= num;
    if day {
        // Dawn ramp in, morning ramp out; the sunset mirrors it around the
        // afternoon (all windows in game ticks).
        sunrise *= remap(time, 0.0, 2700.0, 0.0, 1.0) * remap(time, 3600.0, 10800.0, 1.0, 0.0);
        let s = remap(time, 43200.0, 50400.0, 0.0, 1.0);
        let e = remap(time, 50400.0, 54000.0, 1.0, 0.0);
        sunset *= s * e * e;
    } else {
        sunrise = 0.0;
        sunset = 0.0;
    }
    (sunset, sunrise)
}

/// `NextHorizonRenderer.BlendColor` (`NextHorizonRenderer.cs:183-194`) +
/// `Main.instance.HorizonRenderer.ModifyHorizonLight` (`Main.cs:63921`): the
/// mid gradient entries blend into the sky/tile colour at the visibility
/// strengths — the "HorizonClouds tint".
pub(crate) fn modify_horizon_light(
    color: &mut [f32; 3],
    moon_phase: u8,
    day: bool,
    time: f64,
    atmo: f32,
    cloud_alpha: f32,
) {
    let (sunset, sunrise) = horizon_visibilities(time, day, atmo, cloud_alpha);
    let idx = horizon_phase(moon_phase, day, time) % 4;
    for (channel, gradient) in color.iter_mut().zip(sun_gradients::SUNSETS[idx]) {
        let grad = gradient / 255.0;
        let target = (*channel).max(grad);
        *channel += (target - *channel) * sunset as f32;
    }
    for (channel, gradient) in color.iter_mut().zip(sun_gradients::SUNRISES[idx]) {
        let grad = gradient / 255.0;
        let target = (*channel).max(grad);
        *channel += (target - *channel) * sunrise as f32;
    }
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
    /// Forest far/near mountain layers (`Background_7`/`Background_8`).
    pub(crate) far: Option<BgTex>,
    pub(crate) near: Option<BgTex>,
    /// Forest foreground tree line (`Background_50..52`).
    pub(crate) forest: [Option<BgTex>; 3],
    /// Snow biome: mountains `Background_35/36`, pine line `Background_37..39`.
    pub(crate) snow_far: Option<BgTex>,
    pub(crate) snow_near: Option<BgTex>,
    pub(crate) snow_trees: [Option<BgTex>; 3],
    /// Ocean horizon with islands (`Background_28`, style 0).
    pub(crate) ocean: Option<BgTex>,
    /// Desert dunes (`Background_21` far, `Background_20` near) plus the mountain
    /// layers behind them (`Background_23` far, `Background_22` near).
    pub(crate) desert_far: Option<BgTex>,
    pub(crate) desert_near: Option<BgTex>,
    pub(crate) desert_far_mnt: Option<BgTex>,
    pub(crate) desert_near_mnt: Option<BgTex>,
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
    /// `Images/Moon_0..8` (50x400: eight 50 px phase rows); `Main.moonType` picks
    /// the sheet, `Main.moonPhase` the row.
    pub(crate) moon: [Option<BgTex>; 9],
    pub(crate) stars: Vec<Option<BgTex>>,
    /// W4-f39: `Images/Misc/Sky_Slime_1..4` — the SlimeSky's falling-slime
    /// sheets (`SlimeSky.cs:79-83`), each a 4-row animation strip.
    pub(crate) slime_sky: [Option<BgTex>; 4],
}

/// One star from `Star.SpawnStars` (`Star.cs:86-158`). Positions live in the
/// original's 1920x1200 design space and are mapped onto the screen by `DrawStar`
/// (`Main.cs:63112-63137`); `twinkle` and `rotation` oscillate from their spawn
/// values at the per-star speeds.
#[derive(Clone, Copy)]
pub(crate) struct StarSprite {
    x: f32,
    y: f32,
    scale: f32,
    tex: usize,
    /// `Star.twinkle` at spawn — the phase of the 0.6..1 oscillation.
    phase: f32,
    /// `Star.twinkleSpeed` (per tick, signed).
    twinkle_speed: f32,
    /// `Star.rotation` at spawn (radians).
    rotation: f32,
    /// `Star.rotationSpeed` (per tick, signed).
    rotation_speed: f32,
}

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
    // The five surface variants draw whatever `WorldGen.setBG` rolled for this
    // world (`Main.corruptBG` etc.); `-1` means the C# layer is skipped.
    let layers = |ids: [i32; 3]| {
        ids.map(|i| {
            if i >= 0 {
                sky_tex(atlas, &format!("Background_{i}"), None)
            } else {
                None
            }
        })
    };
    SkyAssets {
        far: sky_tex(atlas, "Background_7", None),
        near: sky_tex(atlas, "Background_8", None),
        forest: [50, 51, 52].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        snow_far: sky_tex(atlas, "Background_35", None),
        snow_near: sky_tex(atlas, "Background_36", None),
        snow_trees: [37, 38, 39].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        ocean: sky_tex(atlas, "Background_28", None),
        desert_far: sky_tex(atlas, "Background_21", None),
        desert_near: sky_tex(atlas, "Background_20", None),
        desert_far_mnt: sky_tex(atlas, "Background_23", None),
        desert_near_mnt: sky_tex(atlas, "Background_22", None),
        storm_far: sky_tex(atlas, "Background_112", None),
        storm_near: sky_tex(atlas, "Background_113", None),
        corrupt_layers: layers(world.corrupt_bg()),
        jungle_layers: layers(world.jungle_bg()),
        hallow_layers: layers(world.hallow_bg()),
        crimson_layers: layers(world.crimson_bg()),
        mushroom_layers: layers(world.mushroom_bg()),
        underground_transition: sky_tex(atlas, "Background_1", None),
        underground: sky_tex(atlas, "Background_2", None),
        rock: sky_tex(atlas, "Background_3", None),
        rock_band: sky_tex(atlas, "Background_4", None),
        magma_transition: sky_tex(atlas, "Background_6", None),
        magma: [125, 126, 127].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        hell_bottom: [185, 186, 187].map(|i| sky_tex(atlas, &format!("Background_{i}"), None)),
        black_tile: sky_tex(atlas, "Black_Tile", None),
        sun: sky_tex(atlas, "Sun", None),
        moon: std::array::from_fn(|i| sky_tex(atlas, &format!("Moon_{i}"), None)),
        stars: (0..4)
            .map(|i| sky_tex(atlas, &format!("Star_{i}"), None))
            .collect(),
        slime_sky: std::array::from_fn(|i| sky_tex(atlas, &format!("Misc/Sky_Slime_{}", i + 1), None)),
    }
}

pub(crate) fn make_stars(world: &World) -> Vec<StarSprite> {
    // `Star.SpawnStars` seeds a `FastRandom.CreateWithRandomSeed()` — a fresh
    // OS-seeded stream per launch (`Star.cs:88`); the port pins it to the world
    // seed so screenshots stay deterministic.
    let mut state = world.seed ^ 0xA076_1D64_78BD_642F;
    let count = next_range(&mut state, 200, 400);
    (0..count)
        .map(|_| {
            // The spawn recipe in its RNG order (`Star.cs:129-152`).
            let x = next_range(&mut state, 0, 1921) as f32;
            let y = next_range(&mut state, 0, 1201) as f32;
            let rotation = next_range(&mut state, 0, 628) as f32 * 0.01;
            let mut scale = next_range(&mut state, 70, 130) as f32 * 0.006;
            let tex = next_range(&mut state, 0, 4) as usize;
            let phase = next_range(&mut state, 60, 101) as f32 * 0.01;
            let mut twinkle_speed = next_range(&mut state, 30, 110) as f32 * 0.0001;
            if next_range(&mut state, 0, 2) == 0 {
                twinkle_speed = -twinkle_speed;
            }
            let mut rotation_speed = next_range(&mut state, 5, 50) as f32 * 0.0001;
            if next_range(&mut state, 0, 2) == 0 {
                rotation_speed = -rotation_speed;
            }
            if next_range(&mut state, 0, 40) == 0 {
                scale *= 2.0;
                twinkle_speed *= 0.5;
                rotation_speed *= 0.5;
            }
            StarSprite {
                x,
                y,
                scale,
                tex,
                phase,
                twinkle_speed,
                rotation,
                rotation_speed,
            }
        })
        .collect()
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

pub(crate) fn draw_sky(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    world: &World,
    cloud_field: &crate::cloud::CloudField,
    sky: &SkyAssets,
    stars: &[StarSprite],
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
    shimmer_alpha: f32,
    // `Main.cloudAlpha` RAW (`Main.cs:63051`'s star gate reads
    // `cloudAlpha * atmo`, NOT the storm deck's cloudBGAlpha). W4-g69
    // retire sweep; g70's tip adds the SAME param for the celestial dims —
    // ONE param at the merge (same name, same semantics).
    cloud_alpha: f32,
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

    // `DrawStarsInBackground` (`Main.cs:63030-63058`): visibility is driven by
    // the sky colour alone — `255 * (1 - cloudAlpha * atmo) - skyR - 25 <= 0`
    // hides them — so the field fades out across dawn instead of switching off
    // at a fixed hour. W4-g69 retire sweep (g69d-star-visual): the heads +
    // gate live in `star_pass_heads` (the RAW cloudAlpha·atmo gate, the remix
    // `||` arm, the graveyard opacity kill, the shimmer sky dim + the
    // shimmerAlpha==1 pass skip).
    let remix = world.seeds.contains(tr_core::seeds::SeedFlags::REMIX);
    let (star_opacity, star_sky, stars_visible) = star_pass_heads(
        tint,
        graveyard_intensity,
        shimmer_alpha,
        cloud_alpha,
        atmo,
        remix,
    );
    if stars_visible {
        for (i, s) in stars.iter().enumerate() {
            let Some(st) = sky.stars.get(s.tex).and_then(|t| *t) else {
                continue;
            };
            // `Star.Update`'s 0.6..1 twinkle oscillation (`Star.cs:215-225`),
            // sine-approximated around the same centre and range.
            let twinkle = 0.8
                + 0.2
                    * (s.phase
                        + anim_time * 60.0 * s.twinkle_speed * (std::f32::consts::TAU / 0.8))
                        .sin();
            // `DrawStar`'s colour chain (`Main.cs:63066-63110`) — the shared
            // helper (the brightness, the graveyard multiply, the shimmer
            // i%3 tint + lerp + dim).
            let Some(rgb) = draw_star_color(star_sky, twinkle, star_opacity, shimmer_alpha, i)
            else {
                continue;
            };
            if rgb[0] == 0 && rgb[1] == 0 && rgb[2] == 0 {
                continue;
            }
            // `Main.cs:63112-63118`: the design-space position mapped onto the
            // screen and shifted by `sceneArea.bgTopY` — which DrawStar zeroes
            // on a skyblock-height world (`worldSurface <= 30`, `:63114-63117`,
            // W4-g69 retire sweep) — then wrapped into the frame (`:63119-63137`,
            // the not-falling arm; the falling-star no-wrap arm is the unported
            // falling-star lane, research/117 §5).
            let star_bg_top = if world.world_surface <= 30.0 { 0.0 } else { bg_top_y };
            let mut px = s.x / 1920.0 * screen.x;
            let mut py = s.y / 1200.0 * screen.y + star_bg_top;
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
            // `Main.cs:63172`: `tex.Size() * scale * twinkle`, rotated about the
            // centre; `Star.Update` advances `rotation` by `rotationSpeed` per
            // tick (`Star.cs:226`). The decompile builds the colour from
            // `default` and assigns only R/G/B (`Main.cs:63083-63086`) — a zero
            // alpha would make every star invisible, so the port draws them
            // fully opaque at the computed brightness (see ANALYSIS.md).
            let k = s.scale * twinkle;
            let rot = s.rotation + anim_time * 60.0 * s.rotation_speed;
            push_screen_rot(
                r,
                camera,
                zoom,
                screen,
                [px, py],
                [st.w * k, st.h * k],
                st.uv,
                [rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0, 1.0],
                rot,
            );
        }
    }

    // `DrawSunAndMoon`: the sun and the moon ride the same arc, anchored to
    // `Main.bgTopY` (`Main.cs:62040`) and drawn only while the camera is near the
    // surface (`Main.cs:62090`). The original computes both bodies every frame but
    // draws only the one matching `Main.dayTime`.
    if view_top / TILE_PX < world.world_surface as f32 + 2.0 {
        let bg_top_y = tr_core::sky::sky_bg_top_y(view_top, world.world_surface);
        // `num12`/`num13`: cloud cover fades the second pass; `Main.atmo` is 1 and
        // the mushroom influence is not modelled.
        let cloud_dim = (1.0 - storm * 1.5).max(0.0);
        if is_day {
            if let Some(sun) = sky.sun {
                let mut body = tr_core::sky::celestial_body(
                    clock_time,
                    tr_core::sky::DAY_LENGTH,
                    bg_top_y,
                    sun.w,
                    screen.x,
                );
                // `num5 *= 1.1f` in the day branch (`Main.cs:62914`).
                body.scale *= 1.1;
                let size = [sun.w * body.scale, sun.h * body.scale];
                // First pass: the raw sun (white — `color = Color.White` unless an
                // eclipse is active, which is not modelled).
                push_screen_rot(
                    r,
                    camera,
                    zoom,
                    screen,
                    [body.x, body.y],
                    size,
                    sun.uv,
                    [1.0, 1.0, 1.0, 1.0],
                    body.rotation,
                );
                // Second pass: `sunColor` at alpha `sunColor.B` (`Main.cs:62941`).
                let c = tr_core::sky::sun_color(clock_time);
                push_screen_rot(
                    r,
                    camera,
                    zoom,
                    screen,
                    [body.x, body.y],
                    size,
                    sun.uv,
                    [
                        c[0] as f32 / 255.0 * cloud_dim,
                        c[1] as f32 / 255.0 * cloud_dim,
                        c[2] as f32 / 255.0 * cloud_dim,
                        c[2] as f32 / 255.0 * cloud_dim,
                    ],
                    body.rotation,
                );
            }
        } else if let Some(moon) = sky.moon[world.moon_type.min(8) as usize] {
            let body = tr_core::sky::celestial_body(
                clock_time,
                tr_core::sky::NIGHT_LENGTH,
                bg_top_y,
                moon.w,
                screen.x,
            );
            // `new Rectangle(0, width * moonPhase, width, width)`: one phase row.
            let phase = world.moon_phase.min(7) as f32;
            let dv = (moon.uv[3] - moon.uv[1]) / 8.0;
            let v0 = moon.uv[1] + dv * phase;
            let uv = [moon.uv[0], v0, moon.uv[2], v0 + dv];
            let c = tr_core::sky::moon_color(clock_time);
            let size = [moon.w * body.scale, moon.w * body.scale];
            push_screen_rot(
                r,
                camera,
                zoom,
                screen,
                [body.x, body.y],
                size,
                uv,
                [
                    c[0] as f32 / 255.0 * cloud_dim,
                    c[1] as f32 / 255.0 * cloud_dim,
                    c[2] as f32 / 255.0 * cloud_dim,
                    cloud_dim,
                ],
                body.rotation,
            );
        }
    }

    // `DrawSurfaceBG_BackMountainsStep1/2`: far then near mountain layer, topped by the
    // biome's tree line. `backgroundTopMagicNumber` makes the layers move at ~30 % of
    // the camera's vertical speed; using the local horizon instead would drag the whole
    // forest down into the view whenever the player is on a hill.
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
    // The `SkyManager` depth tracker (`SkyManager.cs:48-78`), reset at the top
    // of the surface background pass (`Main.cs:59344`). Every `draw_range_min`
    // below ports one `DrawToDepth` call of `Main.cs:59700-60290`; a call
    // whose boundary is above the tracker no-ops, exactly like the original's
    // stale-parallax call (`Main.cs:59890`).
    //
    // DRAW_TO_DEPTH boundary table (`.code-foundations/build/2026-11-03-
    // M1.4-BG-19-ambient-skies-boundary-table.md`):
    //   6.667 = 1/0.15 (Step1, `Main.cs:59812`) — before the far mountains
    //   5.0   (`Main.cs:59878`) — between the far and near mountain rows
    //   1/parallax[i] per front-tree row (`Main.cs:61304+`, biome fns)
    //   1.0   (`Main.cs:60186`) — after the last front row
    //   flush (`Main.cs:59626` `DrawRemainingDepth`) — before the underground
    //   backdrop
    let mut amb_last = f32::INFINITY;
    let amb_ctx = crate::ambience::SkyDrawCtx {
        camera,
        zoom,
        screen,
        color_of_the_skies: sky_color,
        atmo,
        bg_alpha_front,
        background_enabled,
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
    // then 4 (snow). `Background_22/23` are skipped for `desertBG == 4`; our
    // generator never uses that style.
    // Step1's first act is `DrawToDepth(1f/0.15)` (`Main.cs:59812`): the
    // closest entities (EOC slices above depth 6.667) draw behind the
    // mountains.
    // W4-f39: the SlimeSky's slimes ride the same boundary (the C#
    // `SkyManager.DrawDepthRange` walks every active sky; the band is
    // (min, last] BEFORE the tracker moves — the activation order draws
    // Ambience first, Slime second).
    let band_max = amb_last;
    crate::ambience::draw_range_min(
        r,
        ambience,
        lookups,
        &amb_ctx,
        1.0 / 0.15,
        &mut amb_last,
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
    if show_bg {
        if far_forest > 0.0 {
            if let Some(far) = sky.far {
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
                    far_forest,
                );
            }
        }
        if far_desert > 0.0 {
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
                    magic * 1300.0 + 1090.0 + sc_adj + 30.0,
                    false,
                    tint,
                    far_snow,
                );
            }
        }
    }

    // `DrawClouds_Closer` (`Main.cs:59663-59680`): `cY = y * (screenHeight / 600)
    // + bgTopY + 150`. `bgTopY` here is whichever value the last writer left: the
    // mountain Step1 when parallax is on (`Main.cs:59808`), else the plain
    // `bgTopY` of `Main.cs:59328` — the storm banks' own `bgTopY`
    // (`Main.cs:59433`) is overwritten by Step1 before this pass runs.
    // Step1's closing `DrawToDepth(5f)` (`Main.cs:59878`) runs before this
    // pass: the mid-depth entities draw between the mountain rows and the
    // closer clouds.
    let band_max = amb_last;
    crate::ambience::draw_range_min(r, ambience, lookups, &amb_ctx, 5.0, &mut amb_last);
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
    // in vanilla order: forest (0), desert (1), ocean (`Main.oceanBG`,
    // `Background_28`, 3), snow (4).
    if show_bg {
        if far_forest > 0.0 {
            if let Some(near) = sky.near {
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
                    far_forest,
                );
            }
        }
        if far_desert > 0.0 {
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
                draw_bg_layer(
                    r,
                    camera,
                    zoom,
                    screen,
                    &sea,
                    2.3,
                    0.20,
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0,
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
                    magic * 1400.0 + 1260.0 + sc_adj + 30.0,
                    false,
                    tint,
                    far_snow,
                );
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
    let mut amb_pass = show_bg.then_some(crate::render::bg::AmbiencePass {
        entities: ambience,
        lookups,
        ctx: &amb_ctx,
        last: &mut amb_last,
        // W4-c39: the party sky's band state threads the front-row chain
        // (the f40 `lantern_sky`/`lantern_last` shape).
        event_skies,
        party_last: &mut party_last,
        party_surface_px: surface_px,
        slime_sky,
    });
    if show_bg {
        // `pushBGTopHack` for the front loop is `30 - 180` (`Main.cs:59302`,
        // `:59346`, `:59525`): every `top` below is the C# constant minus 150.
        const HACK: f32 = -150.0;
        // Dispatch order is the C# `l` loop's (`Main.cs:59529-59597`).

        // `DrawSurfaceBG_Forest` (`Main.cs:61273-61370`, tree set 1 =
        // `Background_50..52`): bgScale 1.25/1.31/1.34 (C# 61275) x2 (C# 59329);
        // bgParallax 0.40/0.43/0.49; tops 1500/1750/2000 (`Main.cs:61277`).
        // Layer 1 is mirrored horizontally.
        let forest = front_layers(
            [50, 51, 52],
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            sky.forest,
            HACK,
        );
        let forest = [
            forest[0],
            FrontLayer {
                flip: true,
                ..forest[1]
            },
            forest[2],
        ];
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

        // `DrawSurfaceBG_Desert` (`Main.cs:61019-61096`): two dune layers (21/20),
        // parallax 0.37/0.49, scale 1.25/1.34, tops 1750/2150.
        let desert = front_layers(
            [21, 20, -1],
            [
                (1.25, 0.37, 1800.0, 1750.0),
                (1.34, 0.49, 2100.0, 2150.0),
                (1.34, 0.49, 2100.0, 2150.0),
            ],
            [sky.desert_far, sky.desert_near, None],
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

        // `DrawSurfaceBG_Snow` (`Main.cs:60302-60383`, `Main.snowBG` style 1 =
        // `Background_37..39`): same parallax/scale/top as the forest, but no
        // layer is mirrored.
        let snow = front_layers(
            [37, 38, 39],
            [
                (1.25, 0.40, 1800.0, 1500.0),
                (1.31, 0.43, 1950.0, 1750.0),
                (1.34, 0.49, 2100.0, 2000.0),
            ],
            sky.snow_trees,
            HACK,
        );
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
            true,
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
    }

    // `Main.cs:59626` `DrawRemainingDepth`: the shallowest entities (below the
    // last front-row boundary) draw after every surface layer, before the
    // underground backdrop.
    let band_max = amb_last;
    crate::ambience::draw_range_remaining(r, ambience, lookups, &amb_ctx, &mut amb_last);
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
            shimmer_alpha,
        );
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
        // `Main.cs:63792-63829`: midnight (t = 16200) with the ramp floor of 5
        // lifted to the phase minimum — 19 at the full moon, 11 at the new one.
        let full = color_of_the_skies(16200.0, false, 0);
        let empty = color_of_the_skies(16200.0, false, 4);
        let full_byte = full[0] * 255.0;
        let empty_byte = empty[0] * 255.0;
        assert!((full_byte - 19.0).abs() < 1e-4, "full moon {full:?}");
        assert!((empty_byte - 11.0).abs() < 1e-4, "new moon {empty:?}");
        // The right-half mirror: phase 5 quarter floors at 13.
        let quarter = color_of_the_skies(16200.0, false, 5)[0] * 255.0;
        assert!((quarter - 13.0).abs() < 1e-4, "quarter {quarter}");
        // Daylight ignores the floor.
        let day = color_of_the_skies(13500.0, true, 4);
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
        // Night: both closed (`NextHorizonRenderer.cs:231-233`).
        let (sunset, sunrise) = horizon_visibilities(10000.0, false, 1.0, 0.0);
        assert_eq!((sunset, sunrise), (0.0, 0.0));
        // Noon: the dawn window (0..10800) has closed, the dusk one (43200+)
        // has not opened.
        let (sunset, sunrise) = horizon_visibilities(27000.0, true, 1.0, 0.0);
        assert_eq!((sunset, sunrise), (0.0, 0.0));
        // Dawn peak (t=1350): the sunrise at half strength, the sunset closed.
        let (sunset, sunrise) = horizon_visibilities(1350.0, true, 1.0, 0.0);
        assert!(sunset < 1e-6, "sunset={sunset}");
        assert!((sunrise - 0.5).abs() < 1e-6, "sunrise={sunrise}");
        // Dusk peak (t=50400): the sunset at full strength.
        let (sunset, sunrise) = horizon_visibilities(50400.0, true, 1.0, 0.0);
        assert!((sunset - 1.0).abs() < 1e-6, "sunset={sunset}");
        assert!(sunrise < 1e-6, "sunrise={sunrise}");
        // Cloud cover squares into the factor.
        let (sunset_clear, _) = horizon_visibilities(50400.0, true, 1.0, 0.0);
        let (sunset_cloudy, _) = horizon_visibilities(50400.0, true, 1.0, 0.5);
        assert!((sunset_cloudy - sunset_clear * 0.25).abs() < 1e-6);
    }

    #[test]
    fn modify_horizon_light_lifts_dark_dawn_toward_the_gradient() {
        // A dawn-dark sky (25,35,35)/255 with the sunrise at half strength
        // lifts toward Sunrise_Blue[7] (68,144,149)/255 — the max-channel
        // blend (`NextHorizonRenderer.cs:183-194`).
        let mut c = [25.0 / 255.0, 35.0 / 255.0, 35.0 / 255.0];
        modify_horizon_light(&mut c, 1, true, 1350.0, 1.0, 0.0);
        let grad = [68.0 / 255.0, 144.0 / 255.0, 149.0 / 255.0];
        for (got, want) in c.iter().zip(grad) {
            assert!(*got > 0.09 && *got <= want + 1e-6, "got {got}, want ≤ {want}");
        }
        // The white noon colour cannot rise (the max blend + zero visibility).
        let mut noon = [1.0, 1.0, 1.0];
        modify_horizon_light(&mut noon, 1, true, 27000.0, 1.0, 0.0);
        assert_eq!(noon, [1.0, 1.0, 1.0]);
    }

    #[test]
    fn atmosphere_factor_clamps_to_the_unit_range() {
        // Deep underground saturates at 1, high sky bottoms out at 0.
        assert_eq!(atmosphere_factor(3000.0, 300.0, 4200), 1.0);
        assert_eq!(atmosphere_factor(0.0, 300.0, 4200), 0.0);
    }

    #[test]
    fn stars_follow_the_spawn_recipe() {
        // `Star.SpawnStars` (`Star.cs:89`): 200..400 stars, reproducible per
        // seed.
        let mut w = World::new("t".into(), 4200, 1200);
        w.seed = 0xDEAD_BEEF;
        let a = make_stars(&w);
        let b = make_stars(&w);
        assert_eq!(a.len(), b.len());
        assert!((200..400).contains(&a.len()));
        for (sa, sb) in a.iter().zip(b.iter()) {
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
        let noon = color_of_the_skies(27000.0, true, 0);
        assert_eq!(noon, [1.0, 1.0, 1.0], "noon is pure white");
        let morning = color_of_the_skies(6750.0, true, 0);
        let byte = |v: f32| v * 255.0;
        assert!((byte(morning[0]) - 140.0).abs() < 1e-3, "sunrise red");
        assert!((byte(morning[1]) - 145.0).abs() < 1e-3, "sunrise green");
        let sunset = color_of_the_skies(45900.0, true, 0);
        assert!((byte(sunset[0]) - 235.0).abs() < 1e-3, "sunset red");
        assert!((byte(sunset[1]) - 120.0).abs() < 1e-3, "sunset green");
        assert!((byte(sunset[2]) - 170.0).abs() < 1e-3, "sunset blue");
        let dusk = color_of_the_skies(53999.0, true, 0);
        assert!((byte(dusk[0]) - 35.0).abs() < 0.1, "the dusk floor: {dusk:?}");
        assert!(dusk[0] < sunset[0] && sunset[0] < noon[0], "the day darkens");
    }
}
