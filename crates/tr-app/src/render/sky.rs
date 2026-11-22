//! The sky pass: `Main.SetBackColor`'s sky colours, the deterministic cloud/star
//! assets and `DrawSunAndMoon`/`DrawSurfaceBG` (`draw_sky`). The textured
//! underground backdrop it tails into lives in [`super::bg`].
use super::BgTex;
use super::bg::{
    FrontLayer, bg_scale_adjust, draw_background, draw_bg_layer, draw_front_trees,
    forest_173_frame_id, forest_layers, front_layers, layer_y_offset, mushroom_tint,
    push_screen_rot,
};
use crate::app::{next_rand, rand01};
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
    clouds: &[CloudSprite],
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
    // lookups/context their sheets need. Drawn between the passes at the C#
    // `DrawToDepth` boundaries — see the `DRAW_TO_DEPTH` table below.
    ambience: &[crate::ambience::SkyEntity],
    lookups: &crate::assets::Lookups,
    // `Main.ColorOfTheSkies` (the rain-dimmed sky tint, `Main.cs:63209`) and
    // `Main.atmo` — the entity colour chain (`AmbientSky.cs:63`).
    sky_color: [f32; 3],
    atmo: f32,
    // `Main.GameUpdateCount` (`GetBackgroundCounter`, `Main.cs:61264`) — the
    // animated background sheets' frame counter.
    counter: u32,
    // `Main.mouseTextColor` (`Main.cs:1683`): the Mushroom glow's pulse.
    mouse_text_color: u8,
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
    cloud_alpha: f32,
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
    // `Main.shimmerAlpha` — the star dim/recolour (`Main.cs:63047-63050`,
    // `:63087-63110`).
    shimmer_alpha: f32,
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
    // in vanilla order: forest set 1 (0), desert (1), ocean (`Main.oceanBG`,
    // `Background_28`, 3), snow (4), then forest sets 2-4 (10/11/12) — the
    // C# walks 0..6 first and the three extra forest slots last
    // (`Main.cs:59895-59962`).
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
        assert!(dusk[0] < sunset[0] && sunset[0] < noon[0], "the day darkens");
    }
}
