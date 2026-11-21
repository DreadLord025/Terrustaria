//! The background-target CAPTURE/CONSUME seams for the cavern + underworld
//! styles (W4-g72 lane 3, code-only) — the style-specific half the S1-G-13
//! family (d69 `task/W4-d69-layer-target-consumers`, commit `9ba463ae`) does
//! NOT own. UNWIRED from the live frame: the frame-path composition is d69's
//! in-flight consumers lane; the per-biome / Underworld SHEET SOURCES still
//! overflow the single 8192 atlas (ANALYSIS "Deviations kept for the
//! rock/magma passes" / "The hell parallax stack"), so nothing here adds an
//! atlas sheet.
//!
//! C# routing census (the "styles render to target texture, not atlas" claim,
//! VERIFIED — the target is the DESTINATION, the sheets are still
//! `TextureAssets` drawn through `Main.spriteBatch`):
//! - CAVERN: `RenderToTargets` phase 2 renders `backgroundTargetSwap` through
//!   `RenderBackground` (`Main.cs:57290` → `:55200-55218` → `DrawBackground`
//!   `:52798-52853`); `DoDraw` composites `backgroundTarget` with the
//!   cave-parallax X shift (`:62207-62215`). The sheet table is
//!   `_drawBackground_backTexture` (`bg_styles.rs`), the destination is the
//!   `WorldSceneLayerTarget` (`WorldSceneLayerTarget.cs:43-64`).
//! - UNDERWORLD: `DrawBG` (`Main.cs:63973-64063`) runs `DrawSurfaceBG` +
//!   `DrawUnderworldBackground(GameZoomTarget)` (`:64049-64053`); `DoDraw`
//!   binds `screenTarget` via `Filters.Scene.BeginCapture` when `flag3`
//!   (`:62098-62100`, the same gate as `sky_target_active`), else draws to the
//!   backbuffer. `FilterManager.EndCapture` composites `skyTarget` THEN
//!   `screenTarget` into `screenTargetSwap`, both `Color.White` under
//!   `GameViewMatrix.Effects` (`FilterManager.cs:146-152`). So the underworld
//!   stack lands in `screenTarget`, NOT `backgroundTarget` and NOT `skyTarget`
//!   (skyTarget holds only the gradient/stars/sun/moon, `:62025-62095`).
//! - `DrawCapture`'s `!CameraSpaceEffects` arm re-frames the camera onto the
//!   underworld band before `DrawUnderworldBackground()` (`:56844-56862`).
//!
//! Everything here is pure (no RNG, no GPU); the witnesses re-derive the C#
//! math independently. The generic layer composite destination is d69's
//! landed `tr_render::targets::layer_composite_position` — this module gathers
//! its inputs and witnesses the expected result rather than re-implementing it.

use super::BgTex;
use super::bg::push_screen;
use super::underworld_bg::UnderworldLayerPlan;
use glam::Vec2;
use tr_render::SpriteRenderer;

/// The camera `RenderBackground`'s `DrawBackground` sees while it renders into
/// a `WorldSceneLayerTarget` (`WorldSceneLayerTarget.cs:49-57`): the screen
/// shrinks to the target minus the padding, the position snaps to
/// `Utils.Round(center - ScreenSize/2)` (banker's rounding, `Utils.cs:112-115`)
/// and the zoom resets to 1. `draw_background` takes `camera`/`screen` and
/// derives `screen_pos = camera - screen*0.5`, so the adapter hands it the
/// centre of the overridden view; `drawOffset` is the padding because
/// `drawToScreen` is false inside the capture (`Main.cs:52818`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BackgroundCaptureView {
    /// `camera = screen_position + screen_size*0.5` (the overridden centre).
    pub camera: Vec2,
    /// `Main.screenWidth/Height` after the override (`:49-50`).
    pub screen: Vec2,
    /// `Main.GameViewMatrix.Zoom = Vector2.One` (`:52`).
    pub zoom: f32,
    /// `drawOffset = (offScreenRange, offScreenRange)` (`Main.cs:52818`,
    /// `drawToScreen == false`).
    pub draw_offset: Vec2,
}

/// `WorldSceneLayerTarget.UpdateContent`'s screen override (`:49-52`) mapped to
/// `draw_background`'s inputs. `screen_position`/`screen_size` are the d69
/// `LayerCamera` fields (`tr_render::targets::LayerCamera`, the
/// `Utils.Round`ed top-left and the padded size); `offscreen_range` is
/// `Main.offScreenRange` (`family.layout.offscreen_range`).
pub(crate) fn background_capture_view(
    screen_position: [f32; 2],
    screen_size: [f32; 2],
    offscreen_range: f32,
) -> BackgroundCaptureView {
    let screen = Vec2::new(screen_size[0], screen_size[1]);
    BackgroundCaptureView {
        camera: Vec2::new(screen_position[0], screen_position[1]) + screen * 0.5,
        screen,
        zoom: 1.0,
        draw_offset: Vec2::new(offscreen_range, offscreen_range),
    }
}

/// `Main.cs:52818`: `drawOffset = drawToScreen ? Vector2.Zero :
/// (offScreenRange, offScreenRange)`. The live unpadded viewport passes 0 so
/// the pad cancels (ANALYSIS); the capture passes the real range.
pub(crate) fn background_draw_offset(draw_to_screen: bool, offscreen_range: f32) -> Vec2 {
    if draw_to_screen {
        Vec2::ZERO
    } else {
        Vec2::new(offscreen_range, offscreen_range)
    }
}

/// The `DoDraw` background consume dispatch (`Main.cs:62207-62215`):
/// `drawToScreen` draws `DrawBackground()` straight to the screen; otherwise
/// the cached `backgroundTarget` is composited (the cave-parallax X shift,
/// `:62213`). The destination math is d69's `layer_composite_position`; this
/// plan only chooses the arm and carries its inputs.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum BackgroundConsume {
    /// `this.DrawBackground()` (`:62209`).
    Direct,
    /// `spriteBatch.Draw(backgroundTarget.Texture, ...)` (`:62214`) — the
    /// fields are exactly d69 `layer_composite_position(SceneLayer::Background,
    /// position, layer_size, screen_position, camera_center, cave_parallax)`.
    Composite {
        position: [f32; 2],
        layer_size: (u32, u32),
        screen_position: [f32; 2],
        camera_center: [f32; 2],
        cave_parallax: f32,
    },
}

/// `Main.cs:62207-62215` dispatch. `position` is `backgroundTarget.Position`
/// (`WorldSceneLayerTarget.cs:57`, `screenPosition - offScreenRange`),
/// `layer_size` its texture size, `screen_position` `Main.screenPosition`,
/// `camera_center` `Main.Camera.Center`.
pub(crate) fn background_consume(
    draw_to_screen: bool,
    position: [f32; 2],
    layer_size: (u32, u32),
    screen_position: [f32; 2],
    camera_center: [f32; 2],
    cave_parallax: f32,
) -> BackgroundConsume {
    if draw_to_screen {
        BackgroundConsume::Direct
    } else {
        BackgroundConsume::Composite {
            position,
            layer_size,
            screen_position,
            camera_center,
            cave_parallax,
        }
    }
}

/// Where the underworld stack (`DrawBG` → `DrawUnderworldBackground`) renders
/// this frame: `screenTarget` when `flag3` holds (`Main.cs:62098-62100`
/// `BeginCapture(screenTarget)`), else the backbuffer. `flag3` is d69's
/// `sky_target_active(can_capture, foreground_sunlight_effects, draw_to_screen,
/// map_fullscreen, only_draw_fancy_ui, lighting_not_retro)` (`:62021-62022`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum UnderworldCaptureTarget {
    ScreenTarget,
    Backbuffer,
}

pub(crate) fn underworld_capture_target(flag3: bool) -> UnderworldCaptureTarget {
    if flag3 {
        UnderworldCaptureTarget::ScreenTarget
    } else {
        UnderworldCaptureTarget::Backbuffer
    }
}

/// `FilterManager.EndCapture`'s base composite (`FilterManager.cs:146-152`):
/// into `screenTargetSwap`, draw `skyTarget` FIRST then `screenTarget` (the
/// `t` that `BeginCapture` filled), both `Color.White`, both under
/// `GameViewMatrix.Effects` (the gravity flip). The active filters then ping-
/// pong `t`/`t2` (`:154-180`) — out of this lane's scope; the base order is the
/// seam the underworld stack's destination depends on.
pub(crate) fn sky_screen_composite_order() -> [&'static str; 2] {
    ["skyTarget", "screenTarget"]
}

/// The `ugBackTransition` cross-fade overlay tint (`Main.cs:53077-53081`,
/// `:53371-53375`, `:53628-53631`, `:53790-53794`): the OLD style's sheet draws
/// over the new one with every channel (incl. alpha) byte-truncated by
/// `transition`. `base` is the lit cell colour (rgb 0..1, a 1); the C# reads
/// `Lighting.GetColor` bytes, multiplies by `backgroundColor` (0.9) then by
/// `ugBackTransition`, casting to byte each step. The port keeps the engine's
/// 0..1 light and folds the same truncation: `(byte)(c*255*t)/255`.
pub(crate) fn ug_crossfade_tint(base: [f32; 4], transition: f32) -> [f32; 4] {
    let byte = |c: f32| ((c * 255.0) as u8 as f32 * transition) as u8 as f32 / 255.0;
    [byte(base[0]), byte(base[1]), byte(base[2]), byte(base[3])]
}

/// XNA `Rectangle.Intersect` (`Main.cs:56845`): the overlap of two rects, or a
/// zero rect at the origin when they do not overlap.
fn rect_intersect(a: [i32; 4], b: [i32; 4]) -> [i32; 4] {
    let x1 = a[0].max(b[0]);
    let y1 = a[1].max(b[1]);
    let x2 = (a[0] + a[2]).min(b[0] + b[2]);
    let y2 = (a[1] + a[3]).min(b[1] + b[3]);
    if x2 > x1 && y2 > y1 {
        [x1, y1, x2 - x1, y2 - y1]
    } else {
        [0, 0, 0, 0]
    }
}

/// `Utils.Clamp<T>` (`Utils.cs:223-234`): max FIRST, then min — so an inverted
/// band (`min > max`, the small underworld capture) returns `max` when
/// `value > max`, else `min` when `value < min`, else `value`. NOT Rust's
/// `clamp` (which panics on `min > max`).
fn utils_clamp(value: i32, min: i32, max: i32) -> i32 {
    if value > max {
        max
    } else if value < min {
        min
    } else {
        value
    }
}

/// The `DrawCapture` `!CameraSpaceEffects` camera override
/// (`Main.cs:56844-56858`) — the pure re-framing onto the underworld band that
/// precedes `DrawUnderworldBackground()` (`:56861`). `settings_area` is
/// `settings.Area` (tile units), `underworld_layer` is `Main.UnderworldLayer`
/// (`maxTilesY - 200`, `:2790`), `max_world_view_y` is `MaxWorldViewSize.Y`
/// (1200, `:679`). Returns the overridden `screenPosition`/`screenWidth`/
/// `screenHeight`, the `num16` scale and the two transform-translation
/// adjustments (`vector4` is the saved pre-capture `screenPosition`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UnderworldCaptureCamera {
    pub screen_position: [f32; 2],
    pub screen_size: [f32; 2],
    /// `num16 = Max(1f, rectangle2.Height*16f / y2)` (`:56849`).
    pub scale: f32,
    /// `transform.Translation -= (vector4 - rectangle2.TopLeft*16)` (`:56854`).
    pub translation_sub: [f32; 2],
    /// The extra `num16 <= 1f` Y adjustment (`:56855-56858`); 0 otherwise.
    pub translation_sub_y_extra: f32,
}

pub(crate) fn underworld_capture_camera_override(
    settings_area: [i32; 4],
    max_tiles_x: i32,
    max_tiles_y: i32,
    underworld_layer: i32,
    max_world_view_y: i32,
    saved_screen_position: [f32; 2],
) -> UnderworldCaptureCamera {
    // Main.cs:56844: value = (0, UnderworldLayer, maxTilesX, maxTilesY - UnderworldLayer).
    let value = [0, underworld_layer, max_tiles_x, max_tiles_y - underworld_layer];
    // Main.cs:56845: rectangle2 = Intersect(settings.Area, value).
    let r2 = rect_intersect(settings_area, value);
    let y2 = max_world_view_y; // Main.cs:56846
    // Main.cs:56847: Center.Y = r2.Y + r2.Height/2 (integer); Clamp then - y2/2.
    let center_y = r2[1] + r2[3] / 2;
    let screen_y =
        utils_clamp(center_y * 16, value[1] * 16 + y2 / 2, (value[1] + value[3]) * 16 - y2 / 2)
            - y2 / 2;
    // Main.cs:56848: screenPosition.X = rectangle2.Left * 16.
    let screen_x = r2[0] * 16;
    // Main.cs:56849: num16 = Max(1f, (float)r2.Height*16f / (float)y2).
    let num16 = (r2[3] as f32 * 16.0 / y2 as f32).max(1.0);
    // Main.cs:56850-56851: screenHeight = y2; screenWidth = settings.Area.Width*16.
    let screen_h = y2 as f32;
    let screen_w = (settings_area[2] * 16) as f32;
    // Main.cs:56854: Translation -= (vector4 - (r2.X*16, r2.Y*16)).
    let translation_sub = [
        saved_screen_position[0] - (r2[0] * 16) as f32,
        saved_screen_position[1] - (r2[1] * 16) as f32,
    ];
    // Main.cs:56855-56858: if (num16 <= 1f) Translation.Y -= r2.Top*16 - screenPosition.Y.
    let translation_sub_y_extra = if num16 <= 1.0 {
        (r2[1] * 16) as f32 - screen_y as f32
    } else {
        0.0
    };
    UnderworldCaptureCamera {
        screen_position: [screen_x as f32, screen_y as f32],
        screen_size: [screen_w, screen_h],
        scale: num16,
        translation_sub,
        translation_sub_y_extra,
    }
}

/// The underworld layer-plan → sprite adapter: consumes
/// `underworld_bg::underworld_layer_plan`'s output (`UnderworldLayerPlan`) and
/// pushes the tiled sheet + the layer-0 black fill, exactly the
/// `DrawUnderworldBackgroundLayer` loop (`Main.cs:52786-52795`). `tex` is the
/// `TextureAssets.Underworld[num]` sheet handle — `None` until the Underworld
/// sheets land outside the 8192 atlas (a second texture array / target), so the
/// adapter is unwired-safe: it draws nothing without a source. `black_tile_uv`
/// is `TextureAssets.BlackTile` (the (11,3,7) fill, `:52792`).
pub(crate) fn push_underworld_layer(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    plan: &UnderworldLayerPlan,
    tex: Option<&BgTex>,
    black_tile_uv: [f32; 4],
) {
    let Some(tex) = tex else {
        return; // no sheet source yet — the atlas-blocked half
    };
    // The plan's source rect is in sheet pixels; map it onto the atlas UV quad.
    let du = (tex.uv[2] - tex.uv[0]) / tex.w;
    let dv = (tex.uv[3] - tex.uv[1]) / tex.h;
    let uv = [
        tex.uv[0] + du * plan.src[0] as f32,
        tex.uv[1] + dv * plan.src[1] as f32,
        tex.uv[0] + du * (plan.src[0] + plan.src[2]) as f32,
        tex.uv[1] + dv * (plan.src[1] + plan.src[3]) as f32,
    ];
    // Main.cs:52786-52794: i = num11-2 ..= num11+4+num12; each tile draws at
    // `position` scaled by num3, then position.X += num10 (the scaled width).
    let size = [plan.src[2] as f32 * plan.scale, plan.src[3] as f32 * plan.scale];
    let mut x = plan.position[0];
    let y = plan.position[1];
    for _ in plan.first_index - 2..=plan.last_index {
        push_screen(r, camera, zoom, screen, [x, y], size, uv, [1.0, 1.0, 1.0, 1.0]);
        // Main.cs:52789-52793: layer 0 paints the (11,3,7) black fill under each
        // tile, from the tile's scaled bottom to the screen bottom (clamped >= 0).
        if let Some(fill) = plan.black_fill {
            // fill = (x, y, w, h) recomputed per tile X (the Y/size are constant).
            let fy = fill[1] as f32;
            let fh = fill[3].max(0) as f32;
            if fh > 0.0 {
                push_screen(
                    r,
                    camera,
                    zoom,
                    screen,
                    [x, fy],
                    [fill[2] as f32, fh],
                    black_tile_uv,
                    [11.0 / 255.0, 3.0 / 255.0, 7.0 / 255.0, 1.0],
                );
            }
        }
        x += plan.tile_w;
    }
}

#[cfg(test)]
mod bg_target_tests {
    use super::*;

    #[test]
    fn capture_view_centres_the_overridden_screen_and_pads_the_offset() {
        // WorldSceneLayerTarget.cs:49-52: a 2304x1584 target (1920+192*2 x
        // 1200+192*2) overrides the screen to 1920x1200; the Rounded top-left
        // (1000, 2000) centres the camera at (1960, 2600); zoom 1; pad 192.
        let v = background_capture_view([1000.0, 2000.0], [1920.0, 1200.0], 192.0);
        assert_eq!(v.camera, Vec2::new(1960.0, 2600.0));
        assert_eq!(v.screen, Vec2::new(1920.0, 1200.0));
        assert_eq!(v.zoom, 1.0);
        assert_eq!(v.draw_offset, Vec2::new(192.0, 192.0));
    }

    #[test]
    fn draw_offset_is_zero_on_screen_and_padded_in_capture() {
        // Main.cs:52818.
        assert_eq!(background_draw_offset(true, 192.0), Vec2::ZERO);
        assert_eq!(background_draw_offset(false, 192.0), Vec2::new(192.0, 192.0));
    }

    #[test]
    fn consume_dispatch_picks_direct_vs_composite() {
        // Main.cs:62207-62215.
        assert_eq!(
            background_consume(true, [0.0, 0.0], (2304, 1584), [0.0, 0.0], [0.0, 0.0], 0.88),
            BackgroundConsume::Direct
        );
        let c = background_consume(
            false,
            [808.0, 1808.0],
            (2304, 1584),
            [1000.0, 2000.0],
            [1960.0, 2600.0],
            0.88,
        );
        match c {
            BackgroundConsume::Composite { position, layer_size, cave_parallax, .. } => {
                assert_eq!(position, [808.0, 1808.0]);
                assert_eq!(layer_size, (2304, 1584));
                assert_eq!(cave_parallax, 0.88);
            }
            _ => panic!("expected composite"),
        }
    }

    #[test]
    fn composite_destination_rederives_the_cave_parallax_x_shift() {
        // Main.cs:62213-62214 (d69 layer_composite_position, SceneLayer::Background):
        // p = position - screenPosition; p.X += (position.X + size.X/2 - center.X)
        //     * (caveParallax - 1). position (808,1808), size 2304x1584,
        // screenPosition (1000,2000), center (1960,2600), parallax 0.88.
        let position = [808.0f32, 1808.0];
        let size = (2304u32, 1584u32);
        let screen_position = [1000.0f32, 2000.0];
        let center = [1960.0f32, 2600.0];
        let parallax = 0.88f32;
        let mut p = [position[0] - screen_position[0], position[1] - screen_position[1]];
        p[0] += (position[0] + size.0 as f32 / 2.0 - center[0]) * (parallax - 1.0);
        // base (-192, -192); shift (808 + 1152 - 1960) * (-0.12) = 0 * -0.12 = 0.
        assert_eq!(p, [-192.0, -192.0]);
        // A non-centred target shifts X only: position.X 908 -> (908+1152-1960)=100.
        let p2x = (908.0 - 1000.0) + (908.0 + 1152.0 - 1960.0) * (parallax - 1.0);
        assert!((p2x - (-92.0 + 100.0 * -0.12)).abs() < 1e-4);
    }

    #[test]
    fn underworld_routes_to_screen_target_under_flag3() {
        // Main.cs:62098-62100 BeginCapture(screenTarget) iff flag3.
        assert_eq!(underworld_capture_target(true), UnderworldCaptureTarget::ScreenTarget);
        assert_eq!(underworld_capture_target(false), UnderworldCaptureTarget::Backbuffer);
        // FilterManager.cs:150-151: skyTarget first, then screenTarget.
        assert_eq!(sky_screen_composite_order(), ["skyTarget", "screenTarget"]);
    }

    #[test]
    fn crossfade_tint_byte_truncates_every_channel_incl_alpha() {
        // Main.cs:53077-53081: (byte)((float)(int)c * transition). A lit cell
        // (0.5, 0.6, 0.7, 1.0) at transition 0.5: bytes (127,153,178,255) * 0.5
        // -> (63,76,89,127) -> /255.
        let t = ug_crossfade_tint([0.5, 0.6, 0.7, 1.0], 0.5);
        assert_eq!(t[0], 63.0 / 255.0);
        assert_eq!(t[1], 76.0 / 255.0);
        assert_eq!(t[2], 89.0 / 255.0);
        assert_eq!(t[3], 127.0 / 255.0);
        // transition 0 kills the overlay outright.
        assert_eq!(ug_crossfade_tint([0.5, 0.6, 0.7, 1.0], 0.0), [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn rect_intersect_matches_xna_and_empty_when_disjoint() {
        assert_eq!(rect_intersect([0, 0, 10, 10], [5, 5, 10, 10]), [5, 5, 5, 5]);
        assert_eq!(rect_intersect([0, 0, 10, 10], [20, 20, 5, 5]), [0, 0, 0, 0]);
        // touching edges (x2 == x1) are empty.
        assert_eq!(rect_intersect([0, 0, 10, 10], [10, 0, 5, 5]), [0, 0, 0, 0]);
    }

    #[test]
    fn utils_clamp_checks_max_before_min() {
        // Utils.cs:223-234: an inverted band (min 100 > max 50) returns max for
        // value > max, min for value < min, else value — NOT a panic.
        assert_eq!(utils_clamp(200, 100, 50), 50);
        assert_eq!(utils_clamp(20, 100, 50), 100);
        assert_eq!(utils_clamp(75, 100, 50), 75);
        assert_eq!(utils_clamp(5, 0, 10), 5);
        assert_eq!(utils_clamp(15, 0, 10), 10);
        assert_eq!(utils_clamp(-5, 0, 10), 0);
    }

    #[test]
    fn capture_camera_override_reflames_onto_the_underworld_band() {
        // Main.cs:56844-56858. A 4200x1200 world, UnderworldLayer 1000, band
        // value (0,1000,4200,200). settings.Area (100,900,300,400) intersects to
        // (100,1000,300,200). y2 = 1200. Center.Y = 1000+100 = 1100 -> *16 = 17600.
        // Clamp(17600, 1000*16+600=16600, 1200*16-600=18600) = 17600; -600 = 17000.
        // screenX = 100*16 = 1600. num16 = Max(1, 200*16/1200) = Max(1, 2.667) = 2.667.
        // screenH = 1200, screenW = 300*16 = 4800. num16 > 1 so no extra Y adjust.
        let c = underworld_capture_camera_override(
            [100, 900, 300, 400],
            4200,
            1200,
            1000,
            1200,
            [5000.0, 6000.0],
        );
        assert_eq!(c.screen_position, [1600.0, 17000.0]);
        assert_eq!(c.screen_size, [4800.0, 1200.0]);
        assert!((c.scale - 200.0 * 16.0 / 1200.0).abs() < 1e-4);
        assert_eq!(c.translation_sub, [5000.0 - 1600.0, 6000.0 - 16000.0]);
        assert_eq!(c.translation_sub_y_extra, 0.0);
    }

    #[test]
    fn capture_camera_override_small_band_scales_to_one_and_adjusts_y() {
        // A short band: settings.Area (0,1000,4200,50) -> r2 (0,1000,4200,50).
        // num16 = Max(1, 50*16/1200=0.667) = 1. Center.Y = 1000+25 = 1025 -> 16400.
        // Clamp(16400, 16600, 18600): 16400 < min 16600 -> 16600; -600 = 16000.
        // num16 <= 1 -> extra Y = r2.Top*16 - screenY = 16000 - 16000 = 0.
        let c = underworld_capture_camera_override(
            [0, 1000, 4200, 50],
            4200,
            1200,
            1000,
            1200,
            [0.0, 0.0],
        );
        assert_eq!(c.scale, 1.0);
        assert_eq!(c.screen_position, [0.0, 16000.0]);
        assert_eq!(c.translation_sub_y_extra, 0.0);
        // The num16 <= 1 extra-Y arm fires only when the band centre is INSIDE
        // the clamp window but the band is short (height*16 <= y2). A 2000-row
        // world, UnderworldLayer 1800 (band (0,1800,4200,200)), settings.Area
        // (0,1810,4200,60) -> r2 (0,1810,4200,60): Center.Y = 1810+30 = 1840 ->
        // 29440, inside [29400, 31400]; screenY = 29440-600 = 28840. num16 =
        // Max(1, 60*16/1200 = 0.8) = 1 <= 1; extra Y = 1810*16 - 28840 = 120.
        let c2 = underworld_capture_camera_override(
            [0, 1810, 4200, 60],
            4200,
            2000,
            1800,
            1200,
            [0.0, 0.0],
        );
        assert_eq!(c2.scale, 1.0);
        assert_eq!(c2.screen_position, [0.0, 28840.0]);
        assert_eq!(c2.translation_sub_y_extra, 120.0);
    }
}
