//! The sky pass: `Main.SetBackColor`'s sky colours, the deterministic cloud/star
//! assets and `DrawSunAndMoon`/`DrawSurfaceBG` (`draw_sky`). The textured
//! underground backdrop it tails into lives in [`super::bg`].
use super::BgTex;
use super::bg::{
    bg_scale_adjust, draw_background, draw_bg_layer, draw_front_trees, push_screen, push_screen_rot,
};
use crate::{assets, next_rand, rand01};
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
pub(crate) fn color_of_the_skies(time: f64, day: bool) -> [f32; 3] {
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
    } else if time < 16200.0 {
        let t = 1.0 - time / 16200.0;
        let g = byte(t * 30.0 + 5.0).max(15.0 / 255.0);
        [g, g, g]
    } else {
        let t = (time / 32400.0 - 0.5) * 2.0;
        [
            byte(t * 20.0 + 5.0).max(15.0 / 255.0),
            byte(t * 30.0 + 5.0).max(15.0 / 255.0),
            byte(t * 30.0 + 5.0).max(15.0 / 255.0),
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

/// One background cloud. Positions are in a virtual "cloud space": the draw pass
/// adds `camera.x * 0.5`, giving the clouds a slower parallax than the world.
#[derive(Clone, Copy)]
pub(crate) struct CloudSprite {
    uv: [f32; 4],
    w: f32,
    h: f32,
    x: f32,
    /// Fraction of the sky half-height above the camera (0 = center, 1 = top).
    alt: f32,
    scale: f32,
}

/// Deterministic cloud layout for a world. The original keeps a live cloud array;
/// a fixed seeded set is enough for a background layer and keeps the world range
/// reusable while the camera is static.
pub(crate) fn make_clouds(world: &World, lookups: &assets::Lookups) -> Vec<CloudSprite> {
    let available: Vec<assets::CloudTex> = lookups.clouds.iter().flatten().copied().collect();
    if available.is_empty() {
        return Vec::new();
    }
    let mut state = world.seed ^ 0x2545_F491_4F6C_DD1D;
    let span = world.width as f32 * TILE_PX * 0.55 + 3000.0;
    let count = ((span / 300.0) as usize).clamp(40, 400);
    (0..count)
        .map(|_| {
            let tex = available[(next_rand(&mut state) % available.len() as u64) as usize];
            CloudSprite {
                uv: tex.uv,
                w: tex.w,
                h: tex.h,
                x: rand01(&mut state) * span,
                alt: 0.12 + rand01(&mut state) * 0.83,
                scale: 0.45 + rand01(&mut state) * 0.55,
            }
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

#[derive(Clone, Copy)]
pub(crate) struct StarSprite {
    x: f32,
    y: f32,
    size: f32,
    tex: usize,
    phase: f32,
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
    let mut state = world.seed ^ 0xA076_1D64_78BD_642F;
    (0..140)
        .map(|_| StarSprite {
            x: rand01(&mut state),
            // Upper half of the screen; the ground covers the rest.
            y: rand01(&mut state) * 0.62,
            size: 3.0 + rand01(&mut state) * 6.0,
            tex: (next_rand(&mut state) % 4) as usize,
            phase: rand01(&mut state) * std::f32::consts::TAU,
        })
        .collect()
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

    let sky_luma = (tint[0] + tint[1] + tint[2]) / 3.0;
    let star_alpha = ((1.0 - sky_luma * 2.8).clamp(0.0, 1.0)) * 0.85;
    if !is_day && star_alpha > 0.02 {
        for s in stars {
            let Some(st) = sky.stars.get(s.tex).and_then(|t| *t) else {
                continue;
            };
            let twinkle = 0.65 + 0.35 * s.phase.sin();
            push_screen(
                r,
                camera,
                zoom,
                screen,
                [s.x * screen.x, s.y * screen.y],
                [s.size, s.size],
                st.uv,
                [1.0, 1.0, 1.0, star_alpha * twinkle],
            );
        }
    }

    // `DrawSunAndMoon`: the sun and the moon ride the same arc, anchored to
    // `Main.bgTopY` (`Main.cs:62040`) and drawn only while the camera is near the
    // surface (`Main.cs:62090`). The original computes both bodies every frame but
    // draws only the one matching `Main.dayTime`.
    let view_top = camera.y - screen.y * 0.5;
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
    // `Main.screenPosition` is the *unzoomed* view top-left: the surface background
    // is laid out in that space (`DrawSurfaceBG` uses `screenPosition` directly), so
    // changing `GameZoomTarget` must not move or resize it.
    let world_surface_px = (world.world_surface as f32 * TILE_PX).max(1.0);
    let view_top = camera.y - screen.y * 0.5;
    let magic = (300.0 - view_top) / world_surface_px;
    let sc_adj = bg_scale_adjust(world, view_top, screen.y);
    // `Main.DrawSurfaceBG`: `pushBGTopHack = 30` for the mountain passes, then
    // `-180` after the horizon layer, so the front layers use `30 - 180 = -150`.
    // The `+30`/`-150` are folded into the layer tops below.
    // `Main.BackgroundEnabled`: the parallax layers only; the gradient and the
    // clouds above stay, exactly like the original toggle.
    let show_bg = background_enabled && view_top < world_surface_px + TILE_PX;
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

    // Parallax clouds. They are world objects: a fixed altitude above the average
    // surface (so they move with the world, not with the player's height) and a
    // horizontal drift of half the camera plus the wind, like the original's layers.
    let right = camera.x + half.x;
    let cloud_tint = [tint[0] * 0.97, tint[1] * 0.98, tint[2], 1.0];
    for c in clouds {
        let w = c.w * c.scale;
        let h = c.h * c.scale;
        // Vanilla clouds drift with the wind even when the camera is still.
        let wx = c.x + camera.x * 0.5 + anim_time * 4.0;
        let wy = world_surface_px - (60.0 + c.alt * 260.0) - h * 0.5;
        if wx + w < origin.x || wx > right || wy + h < origin.y || wy > bottom {
            continue;
        }
        r.push_back_sprite([wx, wy], [w, h], c.uv, cloud_tint);
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

    // Front layer loop (`Main.DrawSurfaceBG` 59529): forest (0), desert (2), snow (7).
    // Each layer draws with `ColorOfSurfaceBackgroundsBase * bgAlphaFrontLayer[i]`.
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
        );
    }

    // `Main.DrawBackground_SurfaceTransitionBackground`,
    // `Main.DrawBackground_DirtBackground`, `DrawBackground_DrawUnderworldBlackBox`,
    // `DrawBackground_DrawRockLayer` and `DrawBackground_DrawMagmaLayer`: the textured
    // underground backdrop, anchored to `worldSurface` across the whole screen, with
    // the rock bands, the animated magma and the underworld black box below it. (A
    // per-column gradient used to hang a dark "wall" underneath every player-placed
    // row of blocks.)
    static BG: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    if !*BG.get_or_init(|| std::env::var_os("TR_NO_BG").is_some()) {
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
