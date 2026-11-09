//! The sky pass: `Main.SetBackColor`'s sky colours, the deterministic cloud/star
//! assets and `DrawSunAndMoon`/`DrawSurfaceBG` (`draw_sky`). The textured
//! underground backdrop it tails into lives in [`super::bg`].
use super::BgTex;
use super::bg::{
    bg_scale_adjust, draw_background, draw_bg_layer, draw_front_trees, push_screen_rot,
};
use crate::app::{next_rand, rand01};
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

/// `Cloud.GetParallax` (`Cloud.cs:318-338`): the per-cloud fraction of the
/// camera's X movement, from its scale bucket.
fn cloud_parallax(scale: f32) -> f32 {
    let (a, b) = if scale < 1.0 {
        (0.07, (scale + 0.15 + 1.0) / 2.0)
    } else if scale <= 1.15 {
        (0.19, scale - 0.075)
    } else {
        (0.23, scale - 0.15 - 0.075)
    };
    a * (b * b)
}

/// One background cloud from the `Cloud.addCloud` distribution below. `x` is a
/// world-space anchor: the pass renders the cloud `parallax * camera.x` screen
/// pixels to the left of it, the world-space form of `Cloud.UpdateCloudParallax`
/// (`Cloud.cs:295-316`, `position.X -= cameraDelta * GetParallax()`). `y` is the
/// vanilla spawn height in the 600 px reference screen the original spawns in;
/// each pass maps it with its own `DrawClouds_*` formula.
#[derive(Clone, Copy)]
pub(crate) struct CloudSprite {
    uv: [f32; 4],
    w: f32,
    h: f32,
    x: f32,
    y: f32,
    scale: f32,
    /// `Cloud.rotation`, settled into the ±0.02 clamp of `Cloud.Update`
    /// (`Cloud.cs:430-437`).
    rotation: f32,
    /// The `SpriteEffects.FlipHorizontally` coin flip (`Cloud.cs:88-91`).
    flip: bool,
}

/// Deterministic stand-in for the original's live 200-slot cloud array: the
/// attributes of every cloud and the total count follow `Cloud.addCloud`
/// (`Cloud.cs:60-178`) and the clear-weather `numCloudsTemp` equilibrium
/// (`Main.cs:58980-58982`) from the world seed, spread over the world so every
/// camera shows about as many clouds as the original's screen-anchored
/// spawn/kill cycle keeps alive. Deviations (see `docs/ANALYSIS.md`): the wind
/// drift is a constant light wind scaled by `GetParallax` (the original
/// integrates `windSpeedCurrent * 9 * GetParallax` per tick, `Cloud.cs:391`),
/// the spawn-time overlap rejection (`Cloud.cs:170-177`) is not simulated, and
/// the storm type branch (`Cloud.cs:116-127`) and rare clouds
/// (`Cloud.RollRareCloud`, `Cloud.cs:140-147`) need weather/boss state that the
/// static field does not have.
pub(crate) fn make_clouds(world: &World, lookups: &assets::Lookups) -> Vec<CloudSprite> {
    if lookups.clouds.is_empty() {
        return Vec::new();
    }
    let mut state = world.seed ^ 0x2545_F491_4F6C_DD1D;
    // Clear-weather cloud count: the `numCloudsTemp` walk is clamped into
    // 0..=100 while the sky is clear (`Main.cs:58980-58982`) and the
    // `weatherCounter` jumps (`Main.cs:59012-59031`) drive it into that clamp,
    // so the steady state sits at the clamp itself.
    let num_clouds = 100.0f32;
    let span = world.width as f32 * TILE_PX * 0.55 + 3000.0;
    // Anchors spread over `span` (plus the spawn margin) so a 1600 px screen —
    // the original's backbuffer — cuts `num_clouds` of them, matching the
    // original's on-screen density for its `numClouds`.
    let band = span + 1000.0;
    let pool = (num_clouds * band / (1600.0 + 1200.0)).ceil() as usize;
    (0..pool)
        .filter_map(|_| {
            // `Cloud.addCloud` (`Cloud.cs:82-158`) in its RNG order.
            let mut scale = next_range(&mut state, 70, 131) as f32 * 0.01;
            if scale > 1.3 {
                scale = 1.3;
            }
            if scale < 0.7 {
                scale = 0.7;
            }
            // `Cloud.Update` nudges exact 1/1.15 scales out of the bucket edges
            // (`Cloud.cs:383-390`) on its first tick; generate past that so the
            // buckets and the type branches see the steady-state value.
            if scale == 1.0 {
                scale = 0.9999;
            }
            if scale == 1.15 {
                scale = 1.1499;
            }
            let rotation = (next_range(&mut state, -10, 11) as f32 * 0.01).clamp(-0.02, 0.02);
            let flip = next_range(&mut state, 0, 2) == 0;
            let x = rand01(&mut state) * band - 1000.0;
            // Spawn heights in the 600 px reference screen: `-0.25*h .. 0.15*h`,
            // a third of the clouds lifted by up to `0.1*h` (`Cloud.cs:110-114`).
            let mut y = next_range(&mut state, -150, 90) as f32;
            if next_range(&mut state, 0, 3) == 0 {
                y -= next_range(&mut state, 0, 60) as f32;
            }
            let mut kind = next_range(&mut state, 0, 4) as usize;
            // The clear-weather type branches (`Cloud.cs:128-139`), with the
            // screen-height terms at h = 600.
            if scale < 1.0 && y < -90.0 && num_clouds <= 80.0 {
                kind = next_range(&mut state, 9, 14) as usize;
            } else if ((scale < 1.15 && y < -180.0) || (scale < 0.85 && y < 90.0))
                && num_clouds > 70.0
            {
                kind = next_range(&mut state, 4, 9) as usize;
            } else if y > -90.0 && next_range(&mut state, 0, 2) == 0 && num_clouds > 20.0 {
                kind = next_range(&mut state, 14, 18) as usize;
            }
            if scale > 1.2 {
                y += 100.0;
            }
            let tex = lookups.clouds.get(kind).and_then(|t| t.as_ref())?;
            Some(CloudSprite {
                uv: tex.uv,
                w: tex.w,
                h: tex.h,
                x,
                y,
                scale,
                rotation,
                flip,
            })
        })
        .collect()
}

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

pub(crate) fn make_sky_assets(atlas: &Atlas) -> SkyAssets {
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

/// One `DrawClouds_*` pass (`Main.cs:59644-59703`): every cloud of the scale
/// bucket, at `cY = y * screen_y_scale + top`, drawn the way
/// `NextHorizonRenderer.DrawCloud` (`NextHorizonRenderer.cs:241-252`) composes
/// it: `Cloud.cloudColor` (`Cloud.cs:227-235`) scales the sky colour by
/// `scale * Alpha` on every channel including alpha, the distant pass pulls red
/// and green down (`OriginalColorsForCloud`, `NextHorizonRenderer.cs:254-263`),
/// and the sprite rotates about its centre. `bucket` is 1 (scale < 1), 2
/// (1 ..< 1.15) or 3 (>= 1.15) — the `DrawClouds_Distant/Closer/Closest` split.
#[allow(clippy::too_many_arguments)]
fn draw_cloud_pass(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    clouds: &[CloudSprite],
    tint: [f32; 3],
    bucket: u8,
    screen_y_scale: f32,
    top: f32,
    drift: f32,
) {
    for c in clouds {
        let in_bucket = match bucket {
            1 => c.scale < 1.0,
            2 => c.scale >= 1.0 && c.scale < 1.15,
            _ => c.scale >= 1.15,
        };
        if !in_bucket {
            continue;
        }
        let p = cloud_parallax(c.scale);
        let w = c.w * c.scale;
        let h = c.h * c.scale;
        let sx = c.x - p * camera.x + drift * p;
        let cy = c.y * screen_y_scale + top;
        // `Cloud.Update` retires clouds 600 px off the sides (`Cloud.cs:417-420`);
        // the screen-space cull here only saves instances, so a small margin
        // over the ±0.02 rotation is enough.
        if sx + w < -100.0 || sx > screen.x + 100.0 || cy + h < -100.0 || cy > screen.y + 100.0 {
            continue;
        }
        // `Cloud.cloudColor` (`Cloud.cs:227-235`).
        let a = c.scale;
        let mut color = [tint[0] * a, tint[1] * a, tint[2] * a, a];
        if bucket == 1 {
            // `OriginalColorsForCloud` (`NextHorizonRenderer.cs:254-263`).
            color[0] *= c.scale * 0.8;
            color[1] *= (c.scale + 1.0) / 2.0 * 0.9;
        }
        let uv = if c.flip {
            [c.uv[2], c.uv[1], c.uv[0], c.uv[3]]
        } else {
            c.uv
        };
        // `DrawCloud` centres the sprite at the top-left plus half the scaled
        // size and rotates about it (`NextHorizonRenderer.cs:244-251`).
        push_screen_rot(
            r,
            camera,
            zoom,
            screen,
            [sx + w * 0.5, cy + h * 0.5],
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
    clouds: &[CloudSprite],
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
    // The ambient-sky creatures (`AmbientSky`): the live entities and the
    // sheets they sample. Drawn between the passes at the C# `DrawToDepth`
    // boundaries — see the `DRAW_TO_DEPTH` table below.
    ambience: &[crate::ambience::SkyEntity],
    lookups: &crate::assets::Lookups,
    // `Main.ColorOfTheSkies` (the rain-dimmed sky tint, `Main.cs:63209`) and
    // `Main.atmo` — the entity colour chain (`AmbientSky.cs:63`).
    sky_color: [f32; 3],
    atmo: f32,
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
    // at a fixed hour. (`Main.atmo` is 1 at the surface; the graveyard dim of
    // `Main.cs:63037-63045` is not modelled.)
    let sky_r = (tint[0] * 255.0) as i32;
    if 255.0 * (1.0 - storm) - sky_r as f32 - 25.0 > 0.0 {
        for s in stars {
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
            // `DrawStar`'s greyscale brightness (`Main.cs:63069-63080`): how far
            // the sky sits below white, averaged over the channels, x1.4.
            let ch = |c: f32| (255.0 - c * 255.0 - 100.0) * twinkle;
            let n3 = ch(tint[0]) as i32;
            let n4 = ch(tint[1]) as i32;
            let n3 = (n3 + ch(tint[2]) as i32 + n4) / 3;
            if n3 <= 0 {
                continue;
            }
            let mut v = (n3 as f64 * 1.4) as i32;
            if v > 255 {
                v = 255;
            }
            // `Main.cs:63112-63137`: the design-space position mapped onto the
            // screen and shifted by `sceneArea.bgTopY`, wrapped into the frame.
            let mut px = s.x / 1920.0 * screen.x;
            let mut py = s.y / 1200.0 * screen.y + bg_top_y;
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
                [v as f32 / 255.0, v as f32 / 255.0, v as f32 / 255.0, 1.0],
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
    // `DrawSurfaceBG`'s three cloud passes, in the original order: distant
    // (`Main.cs:59334`, before the storm banks), closer (`Main.cs:59449`,
    // between the mountain steps) and closest (`Main.cs:59520`, before the front
    // layers). Clouds live in screen space: `Cloud.UpdateCloudParallax`
    // (`Cloud.cs:295-316`) shifts each cloud against the camera by
    // `Cloud.GetParallax` of the pan, so the screen X is the world anchor minus
    // that fraction of `camera.x`, plus the constant light wind of
    // `make_clouds` scaled the same way (`Cloud.cs:391`).
    let drift = anim_time * 9.0 * 60.0 * 0.1;
    if view_top < world_surface_px + TILE_PX {
        // `DrawClouds_Distant` (`Main.cs:59682-59703`): unscaled cloud Y over
        // `magic * 750 + 830 + scAdj + pushBGTopHack(0)`.
        draw_cloud_pass(
            r,
            camera,
            zoom,
            screen,
            clouds,
            tint,
            1,
            1.0,
            magic * 750.0 + 830.0 + sc_adj,
            drift,
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
    crate::ambience::draw_range_min(
        r,
        ambience,
        lookups,
        &amb_ctx,
        1.0 / 0.15,
        &mut amb_last,
    );
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
    crate::ambience::draw_range_min(r, ambience, lookups, &amb_ctx, 5.0, &mut amb_last);
    if bg_block {
        let closer_top = if background_enabled {
            magic * 1300.0 + 1090.0 + sc_adj + 30.0 + 150.0
        } else {
            magic * 1200.0 + 1190.0 + sc_adj + 150.0
        };
        draw_cloud_pass(
            r,
            camera,
            zoom,
            screen,
            clouds,
            tint,
            2,
            screen.y / 600.0,
            closer_top,
            drift,
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
    if bg_block {
        let closest_base = if background_enabled {
            magic * 1400.0 + 1260.0 + sc_adj + 30.0
        } else {
            magic * 1200.0 + 1190.0 + sc_adj
        };
        draw_cloud_pass(
            r,
            camera,
            zoom,
            screen,
            clouds,
            tint,
            3,
            screen.y / 600.0,
            closest_base * 1.01 - 150.0,
            drift,
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
    });
    if show_bg {
        // `DrawSurfaceBG_Forest` (tree set 1 = `Background_50..52`): parallax
        // 0.40/0.43/0.49, scale 2.5/2.62/2.68, tops +1300/+1550/+1800. The middle
        // layer is mirrored horizontally (`tex == 51`).
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &sky.forest,
            front_forest,
            [2.5, 2.62, 2.68],
            [0.40, 0.43, 0.49],
            [1800.0, 1950.0, 2100.0],
            [1300.0, 1550.0, 1800.0],
            Some(1),
            magic,
            sc_adj,
            tint,
            amb_pass.as_mut(),
            false,
        );
        // `DrawSurfaceBG_Desert`: two dune layers (21/20), parallax 0.37/0.49,
        // scale 2.5/2.68, tops +1600/+2000, no tree line.
        let desert_trees = [sky.desert_far, sky.desert_near, None];
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &desert_trees,
            front_desert,
            [2.5, 2.68, 2.68],
            [0.37, 0.49, 0.49],
            [1800.0, 2100.0, 2100.0],
            [1600.0, 2000.0, 2000.0],
            None,
            magic,
            sc_adj,
            tint,
            amb_pass.as_mut(),
            false,
        );
        // `DrawSurfaceBG_Snow` (tree set `Main.snowBG` style 1 = `Background_37..39`):
        // same parallax/scale/top as the forest, but none of the layers is mirrored
        // and there is no -50 texture tweak.
        draw_front_trees(
            r,
            camera,
            zoom,
            screen,
            &sky.snow_trees,
            front_snow,
            [2.5, 2.62, 2.68],
            [0.40, 0.43, 0.49],
            [1800.0, 1950.0, 2100.0],
            [1350.0, 1600.0, 1850.0],
            None,
            magic,
            sc_adj,
            tint,
            amb_pass.as_mut(),
            true,
        );
    }

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
        let small = cloud_parallax(0.9999);
        let mid = cloud_parallax(1.1499);
        let big = cloud_parallax(1.3);
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
    fn clouds_stay_within_the_vanilla_scale_clamps() {
        let mut w = World::new("t".into(), 4200, 1200);
        w.seed = 7;
        // The vanilla pool types 0..21 (`Cloud_0..21`); a synthetic entry per
        // type exercises the whole `addCloud` generation path.
        let mut lookups = assets::Lookups::default();
        lookups.clouds = (0..22)
            .map(|_| {
                Some(assets::CloudTex {
                    uv: [0.0; 4],
                    w: 100.0,
                    h: 50.0,
                })
            })
            .collect();
        let clouds = make_clouds(&w, &lookups);
        assert!(!clouds.is_empty());
        for c in &clouds {
            assert!((0.7..=1.3).contains(&c.scale));
            assert!(c.rotation.abs() <= 0.02);
            // The 600 px reference-screen spawn band plus the >1.2 lift
            // (`Cloud.cs:110-114`, `Cloud.cs:148-151`).
            assert!((-360.0..=190.0).contains(&c.y));
        }
        // Deterministic per seed.
        let again = make_clouds(&w, &lookups);
        assert_eq!(clouds.len(), again.len());
        assert_eq!(clouds[0].x, again[0].x);
    }

    #[test]
    fn empty_cloud_textures_yield_an_empty_field() {
        let mut w = World::new("t".into(), 4200, 1200);
        w.seed = 7;
        let clouds = make_clouds(&w, &assets::Lookups::default());
        assert!(clouds.is_empty());
    }
}
