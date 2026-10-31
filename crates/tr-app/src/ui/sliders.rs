//! The settings slider chrome: `IngameOptions.DrawValueBar`
//! (`IngameOptions.cs:1537-1580`) and the `Main.DrawMenu` HSL colour-slider block
//! (`Main.cs:51102-51284`) — the shared controls of the volume / parallax screens
//! and the cursor / border colour screens.
//!
//! Like the rest of `ui/`, every draw reads the sheets from `Lookups` `Option`s: a
//! missing sheet draws nothing, so `Lookups::default()` renders a blank frame.
//! The drag state is mutated inside the same pass that draws (Review Focus 1):
//! [`draw_color_sliders`] runs [`color_sliders_update`] first and renders its
//! results, so a knob can never desync from the value it stands for.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::color;
use crate::assets;

/// `TextureAssets.ColorBar` (`IngameOptions.cs:1539-1540`), decoded 178x16 — the
/// value-bar track and the sat/lum/alpha rows of the colour menus.
pub const BAR_W: f32 = 178.0;
pub const BAR_H: f32 = 16.0;
/// `TextureAssets.ColorBlip` (`IngameOptions.cs:1552-1555`), decoded 2x8 — one
/// gradient segment.
pub const BLIP_W: f32 = 2.0;
pub const BLIP_H: f32 = 8.0;
/// `TextureAssets.ColorSlider` (`IngameOptions.cs:1566`), decoded 11x25 — the knob.
pub const KNOB_W: f32 = 11.0;
pub const KNOB_H: f32 = 25.0;
/// `IngameOptions.cs:1549` (`int num = 167`) and `Main.cs:51121` (`num54 = 167`).
pub const BLIPS: i32 = 167;
/// `Main.cs:51146` — the knob rides `(Hue.Width() - 2) * bar` on the HSL sliders.
pub const HUE_TRAVEL: f32 = 176.0; // Hue.Width() - 2

/// `Main.OurFavoriteColor = new Color(255, 231, 69)` (`Main.cs:66748`): the
/// hover-highlight tint.
const OUR_FAVORITE_COLOR: [u8; 4] = [255, 231, 69, 255];

/// XNA `Color.Lerp(Color.Black, Color.White, percent)` (`Utils.cs:102-105`):
/// every channel `(byte)(255f * percent)` **truncated**, alpha 255.
pub fn color_lerp_black_to_white(percent: f32) -> [u8; 4] {
    // C#'s `(byte)` of a float wraps modulo 256 (ECMA-335 `conv.u1`), so the
    // emulation casts through a signed integer like `color::mul`.
    let c = ((255.0f32 * percent) as i64) as u8;
    [c, c, c, 255]
}

/// `IngameOptions.DrawValueBar` geometry (`IngameOptions.cs:1541-1543`): the
/// bar's right edge sits on `right_x`, vertically centred on `center_y`.
/// Returns `(rect, hit)` — `hit` is `rectangle.Inflate((int)(-5f*scale), 0)`
/// (`IngameOptions.cs:1556`).
pub fn value_bar_rects(right_x: f32, center_y: f32) -> ([f32; 4], [f32; 4]) {
    let rect = [right_x - BAR_W, center_y - BAR_H / 2.0, BAR_W, BAR_H];
    let hit = [rect[0] + 5.0, rect[1], rect[2] - 10.0, rect[3]];
    (rect, hit)
}

/// `IngameOptions.cs:1567-1579`: `(mouseX - rect.X) / rect.Width` over the bar
/// width; the mouse left of the bar reads 0, right of it 1. (The plan pins the
/// *uninflated* `rect` here; the C# divides by the already-`Inflate`d rectangle —
/// same clamps, slightly different slope off centre.)
pub fn value_bar_value(rect: [f32; 4], mouse_x: f32) -> f32 {
    ((mouse_x - rect[0]) / rect[2]).clamp(0.0, 1.0)
}

/// Knob centre (`IngameOptions.cs:1566`, drawn with the sheet's centre as
/// origin): `(rect.x + 5 + 167*value, rect.y + 8)`.
pub fn value_bar_knob(rect: [f32; 4], value: f32) -> [f32; 2] {
    [rect[0] + 5.0 + BLIPS as f32 * value, rect[1] + 8.0]
}

/// Draws one value bar: the ColorBar sheet, `BLIPS+1` ColorBlips in
/// `color_lerp_black_to_white(i/167)` (the plan pins the inclusive count; the C#
/// loop at `IngameOptions.cs:1552-1555` runs `num4 < 167`), the
/// `Slider_Highlight` overlay tinted `OurFavoriteColor` while the mouse is inside
/// the hit box (`IngameOptions.cs:1557-1564`), then the ColorSlider knob.
// Consumed by the volume / parallax screens (Task 2).
#[allow(dead_code)]
pub fn draw_value_bar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    right_x: f32,
    center_y: f32,
    value: f32,
    mouse: Vec2,
) {
    let (rect, hit) = value_bar_rects(right_x, center_y);
    // `sb.Draw(value, rectangle, Color.White)` (`IngameOptions.cs:1548`).
    if let Some(uv) = lookups.hud_uv(lookups.ui_color_bar.as_ref()) {
        r.push_ui_sprite([rect[0], rect[1]], [rect[2], rect[3]], uv, [1.0; 4]);
    }
    // The gradient blips (`IngameOptions.cs:1550-1555`): one per step, starting
    // 5 px in and 4 px down.
    if let Some(uv) = lookups.hud_uv(lookups.ui_color_blip.as_ref()) {
        for i in 0..=BLIPS {
            let c = color_lerp_black_to_white(i as f32 / BLIPS as f32);
            r.push_ui_sprite(
                [rect[0] + 5.0 + i as f32, rect[1] + 4.0],
                [BLIP_W, BLIP_H],
                uv,
                color::tint(c),
            );
        }
    }
    // `TextureAssets.ColorHighlight` over the whole bar while hovered
    // (`IngameOptions.cs:1557-1564`).
    if inside(hit, mouse) {
        if let Some(uv) = lookups.hud_uv(lookups.ui_slider_highlight.as_ref()) {
            r.push_ui_sprite(
                [rect[0], rect[1]],
                [rect[2], rect[3]],
                uv,
                color::tint(OUR_FAVORITE_COLOR),
            );
        }
    }
    // `TextureAssets.ColorSlider`, centre origin (`IngameOptions.cs:1566`).
    if let Some(uv) = lookups.hud_uv(lookups.ui_color_slider.as_ref()) {
        let centre = value_bar_knob(rect, value);
        r.push_ui_sprite(
            [centre[0] - KNOB_W / 2.0, centre[1] - KNOB_H / 2.0],
            [KNOB_W, KNOB_H],
            uv,
            [1.0; 4],
        );
    }
}

/// The four HSL sliders of the colour menus (`Main.DrawMenu`, `Main.cs:51102-51284`).
/// `h`/`s`/`l`/`a` mirror `Main.hBar`/`sBar`/`lBar`/`aBar`; the `-1f` "unseeded"
/// sentinel and the `rgbToHsl(selColor)` seeding live with the menu state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HslBars {
    pub h: f32,
    pub s: f32,
    pub l: f32,
    pub a: f32,
}

/// Bar tops: `num9`, `num9+26`, `num9+52`, `num9+78` (`Main.cs:51118`, `:51167`,
/// `:51199`, `:51239`). Index 3 is the alpha bar (menuMode 252 only).
pub fn color_slider_tops(top: f32) -> [f32; 4] {
    [top, top + 26.0, top + 52.0, top + 78.0]
}

/// Bar left edge: `screenWidth/2 - Hue.Width()/2` (`Main.cs:51120`).
pub fn color_slider_left(screen_w: f32) -> f32 {
    screen_w / 2.0 - BAR_W / 2.0
}

/// Hit box: the bar rect inflated by 4 px on every side (`Main.cs:51142` and the
/// row twins: `mouseX > num53 - 4 && mouseX < num53 + Hue.Width() + 4 && ...`).
pub fn color_slider_hit(left: f32, top: f32) -> [f32; 4] {
    [left - 4.0, top - 4.0, BAR_W + 8.0, BAR_H + 8.0]
}

/// Knob top-left (`Main.cs:51146` and the sBar/lBar/aBar twins): the 3-arg
/// `SpriteBatch.Draw` has a zero origin, so
/// `(left + HUE_TRAVEL*t - KNOB_W/2, top - KNOB_H/2 + BAR_H/2)`.
pub fn color_slider_knob(left: f32, top: f32, t: f32) -> [f32; 2] {
    [
        left + HUE_TRAVEL * t - KNOB_W / 2.0,
        top - KNOB_H / 2.0 + BAR_H / 2.0,
    ]
}

/// XNA `Rectangle.Contains(Point)` (`IngameOptions.cs:1557`): inclusive left/top,
/// exclusive right/bottom.
fn inside(rect: [f32; 4], p: Vec2) -> bool {
    p.x >= rect[0] && p.x < rect[0] + rect[2] && p.y >= rect[1] && p.y < rect[1] + rect[3]
}

/// Whether one colour-slider row takes the mouse: it hovers the row's hit box
/// while no grab is held, or it holds the grab ((hover && grab == 0) || grab ==
/// id — `Main.cs:51142`, `:51177`, `:51212`, `:51253`).
fn bar_active(left: f32, top: f32, id: u8, mouse: Vec2, grab: u8) -> bool {
    (inside(color_slider_hit(left, top), mouse) && grab == 0) || grab == id
}

/// The dragged 0..1 value of a row: `(mouseX - num53) / Hue.Width()`, clamped
/// (`Main.cs:51154-51160` and the row twins).
fn bar_value(left: f32, mouse_x: f32) -> f32 {
    ((mouse_x - left) / BAR_W).clamp(0.0, 1.0)
}

/// The draw-time drag half of the HSL block (`Main.cs:51102-51284`) as one pure
/// pass, so [`draw_color_sliders`] keeps hit-test → mutate in the same pass that
/// draws (Review Focus 1) and the grab semantics stay testable without a GPU.
///
/// Releasing the button clears the sticky grab (`Main.cs:51112-51115`); while it
/// is down, an active row takes the grab (1 hue, 2 sat, 3 lum, 4 alpha) and
/// follows the mouse. The luminosity grab re-floors the stored value with
/// `lum_floor` — 0.15 for menuMode 25, 0.0 for 252 (`Main.cs:51201-51202`):
/// `l = t * (1 - floor) + floor` (`Main.cs:51232`). Returns the four
/// `Slider_Highlight` flags.
#[allow(clippy::too_many_arguments)]
fn color_sliders_update(
    screen_w: f32,
    top: f32,
    bars: &mut HslBars,
    lum_floor: f32,
    alpha_bar: bool,
    mouse: Vec2,
    mouse_down: bool,
    grab: &mut u8,
) -> [bool; 4] {
    let left = color_slider_left(screen_w);
    let tops = color_slider_tops(top);
    if !mouse_down {
        // `if (!Main.mouseLeft) { this.grabColorSlider = 0; ... }` (`:51112-51115`).
        *grab = 0;
    }
    let mut highlight = [false; 4];

    // Hue — grab 1 (`Main.cs:51142-51163`).
    highlight[0] = bar_active(left, tops[0], 1, mouse, *grab);
    if highlight[0] && mouse_down {
        *grab = 1;
        bars.h = bar_value(left, mouse.x);
    }
    // Saturation — grab 2 (`Main.cs:51177-51195`).
    highlight[1] = bar_active(left, tops[1], 2, mouse, *grab);
    if highlight[1] && mouse_down {
        *grab = 2;
        bars.s = bar_value(left, mouse.x);
    }
    // Luminosity — grab 3, re-floored (`Main.cs:51212-51232`).
    highlight[2] = bar_active(left, tops[2], 3, mouse, *grab);
    if highlight[2] && mouse_down {
        *grab = 3;
        bars.l = bar_value(left, mouse.x) * (1.0 - lum_floor) + lum_floor;
    }
    // Alpha — grab 4, only while the alpha bar shows (`Main.cs:51253-51269`).
    if alpha_bar {
        highlight[3] = bar_active(left, tops[3], 4, mouse, *grab);
        if highlight[3] && mouse_down {
            *grab = 4;
            bars.a = bar_value(left, mouse.x);
        }
    }
    highlight
}

/// Draws the hue bar, the saturation and luminosity blip bars and (when
/// `alpha_bar`) the alpha bar whose blips are `hsl_to_rgb(h,s,l,255) * (i/167)`
/// (`Main.cs:51246`, XNA `Color * float` truncation -> `color::mul`).
/// Hover/grab mirror `grabColorSlider` (`Main.cs:51102-51284`): `grab` holds
/// 1 hue, 2 sat, 3 lum, 4 alpha while the button is down; 0 otherwise.
/// `lum_floor` is 0.15 for menuMode 25 and 0.0 for 252 (`Main.cs:51201-51202`):
/// the knob maps `(l - floor) / (1 - floor)` and a grab stores
/// `l = t * (1 - floor) + floor`.
// Consumed by the cursor / border colour screens (Task 3).
#[allow(dead_code)]
#[allow(clippy::too_many_arguments)]
pub fn draw_color_sliders(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen_w: f32,
    top: f32,
    bars: &mut HslBars,
    lum_floor: f32,
    alpha_bar: bool,
    mouse: Vec2,
    mouse_down: bool,
    grab: &mut u8,
) {
    let highlight =
        color_sliders_update(screen_w, top, bars, lum_floor, alpha_bar, mouse, mouse_down, grab);
    let left = color_slider_left(screen_w);
    let tops = color_slider_tops(top);
    let hl = color::tint(OUR_FAVORITE_COLOR);
    let knob_uv = lookups.hud_uv(lookups.ui_color_slider.as_ref());
    let bar_uv = lookups.hud_uv(lookups.ui_color_bar.as_ref());
    let blip_uv = lookups.hud_uv(lookups.ui_color_blip.as_ref());
    let highlight_uv = lookups.hud_uv(lookups.ui_slider_highlight.as_ref());

    // Hue row (`Main.cs:51141-51146`): the `Hue` gradient, no blips.
    if let Some(uv) = lookups.hud_uv(lookups.ui_hue.as_ref()) {
        r.push_ui_sprite([left, tops[0]], [BAR_W, BAR_H], uv, [1.0; 4]);
    }
    if highlight[0] {
        if let Some(uv) = highlight_uv {
            r.push_ui_sprite([left, tops[0]], [BAR_W, BAR_H], uv, hl);
        }
    }
    if let Some(uv) = knob_uv {
        let k = color_slider_knob(left, tops[0], bars.h);
        r.push_ui_sprite([k[0], k[1]], [KNOB_W, KNOB_H], uv, [1.0; 4]);
    }

    // Saturation row (`Main.cs:51167-51178`): ColorBar + blips through the
    // saturation axis, `hslToRgb(h, i/167, l)`.
    draw_blip_bar(
        r,
        bar_uv,
        blip_uv,
        highlight_uv,
        hl,
        left,
        tops[1],
        highlight[1],
        bars.s,
        knob_uv,
        |i| color::hsl_to_rgb(bars.h as f64, blip_step(i), bars.l as f64, 255),
    );

    // Luminosity row (`Main.cs:51199-51215`): blips through the luminosity axis,
    // `hslToRgb(h, s, i/167)`; the knob maps the floored range.
    let knob_t = (bars.l - lum_floor) / (1.0 - lum_floor);
    draw_blip_bar(
        r,
        bar_uv,
        blip_uv,
        highlight_uv,
        hl,
        left,
        tops[2],
        highlight[2],
        knob_t,
        knob_uv,
        |i| color::hsl_to_rgb(bars.h as f64, bars.s as f64, blip_step(i), 255),
    );

    // Alpha row (`Main.cs:51239-51252`, menuMode 252 only): `color6 * (i/167)`.
    if alpha_bar {
        draw_blip_bar(
            r,
            bar_uv,
            blip_uv,
            highlight_uv,
            hl,
            left,
            tops[3],
            highlight[3],
            bars.a,
            knob_uv,
            |i| {
                color::mul(
                    color::hsl_to_rgb(bars.h as f64, bars.s as f64, bars.l as f64, 255),
                    blip_step(i) as f32,
                )
            },
        );
    }
}

/// `(float)num / (float)num54` of the C# blip loops, widened to the `double`
/// `hslToRgb` parameter exactly as the call sites do.
fn blip_step(i: i32) -> f64 {
    (i as f32 / BLIPS as f32) as f64
}

/// One blip-bar row (`Main.cs:51167-51178` and the twins): the ColorBar sheet,
/// the `0..=167` gradient blips 5 px in / 4 px down, the highlight overlay while
/// active, then the knob. `blip_color` colors the i-th blip.
#[allow(clippy::too_many_arguments)]
fn draw_blip_bar(
    r: &mut SpriteRenderer,
    bar_uv: Option<[f32; 4]>,
    blip_uv: Option<[f32; 4]>,
    highlight_uv: Option<[f32; 4]>,
    hl: [f32; 4],
    left: f32,
    top: f32,
    highlight: bool,
    knob_t: f32,
    knob_uv: Option<[f32; 4]>,
    blip_color: impl Fn(i32) -> [u8; 4],
) {
    if let Some(uv) = bar_uv {
        r.push_ui_sprite([left, top], [BAR_W, BAR_H], uv, [1.0; 4]);
    }
    if let Some(uv) = blip_uv {
        for i in 0..=BLIPS {
            r.push_ui_sprite(
                [left + 5.0 + i as f32, top + 4.0],
                [BLIP_W, BLIP_H],
                uv,
                color::tint(blip_color(i)),
            );
        }
    }
    if highlight {
        if let Some(uv) = highlight_uv {
            r.push_ui_sprite([left, top], [BAR_W, BAR_H], uv, hl);
        }
    }
    if let Some(uv) = knob_uv {
        let k = color_slider_knob(left, top, knob_t);
        r.push_ui_sprite([k[0], k[1]], [KNOB_W, KNOB_H], uv, [1.0; 4]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values derived from `IngameOptions.cs:1537-1580` /
    // `Main.cs:51102-51284` / `Utils.cs:102-105` — never from this code.

    #[test]
    fn color_lerp_black_to_white_truncates_every_channel() {
        assert_eq!(color_lerp_black_to_white(0.0), [0, 0, 0, 255]);
        // 255 * (84/167) = 128.26 -> (byte) 128: truncated, not rounded.
        assert_eq!(color_lerp_black_to_white(84.0 / 167.0), [128, 128, 128, 255]);
        assert_eq!(color_lerp_black_to_white(1.0), [255, 255, 255, 255]);
    }

    #[test]
    fn value_bar_rects_match_the_ingame_options_geometry() {
        // valuePosition.X -= width, Y - height/2 (`IngameOptions.cs:1541-1543`);
        // hit = Inflate(-5, 0) (`:1556`).
        let (rect, hit) = value_bar_rects(797.0, 332.0);
        assert_eq!(rect, [619.0, 324.0, 178.0, 16.0]);
        assert_eq!(hit, [624.0, 324.0, 168.0, 16.0]);
    }

    #[test]
    fn value_bar_value_divides_over_the_bar_width_and_clamps() {
        let (rect, _) = value_bar_rects(797.0, 332.0);
        assert_eq!(value_bar_value(rect, 619.0), 0.0);
        assert_eq!(value_bar_value(rect, 708.0), 0.5, "708 = 619 + 89");
        assert_eq!(value_bar_value(rect, 797.0), 1.0);
        // Positional clamp (`IngameOptions.cs:1573-1579`).
        assert_eq!(value_bar_value(rect, 100.0), 0.0);
        assert_eq!(value_bar_value(rect, 1000.0), 1.0);
    }

    #[test]
    fn value_bar_knob_rides_the_167_blip_travel() {
        let (rect, _) = value_bar_rects(797.0, 332.0);
        // Centre = (rect.x + 5 + 167*0.5, rect.y + 8) = 619 + 5 + 83.5.
        // (The plan table's literal "[703.0, 332.0]" contradicts its own
        // "619+5+83.5" arithmetic and the cited Draw call; 707.5 is the value
        // both agree on.)
        assert_eq!(value_bar_knob(rect, 0.5), [707.5, 332.0]);
    }

    #[test]
    fn color_slider_geometry_matches_the_hsl_block() {
        // `Main.cs:51118` (`num52 = num9`) and the `+= 26` steps (`:51167`,
        // `:51199`, `:51239`).
        assert_eq!(color_slider_tops(320.0), [320.0, 346.0, 372.0, 398.0]);
        // `Main.cs:51120`: screenWidth/2 - Hue.Width()/2 = 800 - 89.
        assert_eq!(color_slider_left(1600.0), 711.0);
        // `Main.cs:51146` with float halves (the C# divides the int sheet sizes,
        // which lands half a pixel higher/left — see the report).
        assert_eq!(color_slider_knob(711.0, 320.0, 0.5), [793.5, 315.5]);
        // `Main.cs:51142`: the hit box inflates by 4 on every side.
        assert_eq!(color_slider_hit(711.0, 320.0), [707.0, 316.0, 186.0, 24.0]);
    }

    #[test]
    fn slider_constants_mirror_the_sheets() {
        assert_eq!((BAR_W, BAR_H), (178.0, 16.0));
        assert_eq!((BLIP_W, BLIP_H), (2.0, 8.0));
        assert_eq!((KNOB_W, KNOB_H), (11.0, 25.0));
        assert_eq!(BLIPS, 167);
        assert_eq!(HUE_TRAVEL, 176.0, "Hue.Width() - 2");
    }

    #[test]
    fn luminosity_grab_refloors_and_the_knob_maps_back() {
        let mut bars = HslBars { h: 0.3, s: 0.6, l: 0.2, a: 1.0 };
        let before = bars;
        let mut grab: u8 = 0;
        let left = color_slider_left(1600.0);
        let tops = color_slider_tops(320.0);
        // Mouse at t = 0.5 inside the luminosity row's hit box, button down.
        let mouse = Vec2::new(left + 0.5 * BAR_W, tops[2] + BAR_H / 2.0);
        let hl = color_sliders_update(1600.0, 320.0, &mut bars, 0.15, false, mouse, true, &mut grab);
        assert_eq!(grab, 3);
        assert!(hl[2]);
        // `l = 0.5 * (1 - 0.15) + 0.15 = 0.575` (`Main.cs:51232`).
        assert!((bars.l - 0.575).abs() < 1e-6, "l = {}", bars.l);
        // The knob maps (0.575 - 0.15) / 0.85 back to t = 0.5.
        let t = (bars.l - 0.15) / (1.0 - 0.15);
        assert!((t - 0.5).abs() < 1e-6);
        assert_eq!(
            color_slider_knob(left, tops[2], t),
            color_slider_knob(left, tops[2], 0.5)
        );
        // The other bars were not touched.
        assert_eq!((bars.h, bars.s, bars.a), (before.h, before.s, before.a));
    }

    #[test]
    fn hue_grab_stores_the_clamped_mouse_value() {
        let mut bars = HslBars { h: 0.0, s: 0.0, l: 0.0, a: 0.0 };
        let mut grab: u8 = 0;
        let left = color_slider_left(1600.0);
        let tops = color_slider_tops(320.0);
        // Grab from inside the row, at t = 0.5.
        let mouse = Vec2::new(left + 89.0, tops[0] + BAR_H / 2.0);
        color_sliders_update(1600.0, 320.0, &mut bars, 0.15, false, mouse, true, &mut grab);
        assert_eq!(grab, 1);
        assert_eq!(bars.h, 0.5);
        // Sliding past the right edge while held clamps to 1
        // (`Main.cs:51154-51161`, the grab sticks outside the hit box).
        let mouse = Vec2::new(left + BAR_W + 50.0, tops[0] + BAR_H / 2.0);
        color_sliders_update(1600.0, 320.0, &mut bars, 0.15, false, mouse, true, &mut grab);
        assert_eq!(bars.h, 1.0);
        // The alpha row is out of reach without its bar (menuMode 25): the
        // sticky hue grab keeps holding.
        let mouse = Vec2::new(left + 89.0, tops[3] + BAR_H / 2.0);
        color_sliders_update(1600.0, 320.0, &mut bars, 0.15, false, mouse, true, &mut grab);
        assert_eq!(grab, 1, "grab stays on the hue row");
        assert_eq!(bars.a, 0.0);
        // Released, then pressed again with the alpha bar shown (menuMode 252):
        // the same spot grabs row 4.
        color_sliders_update(1600.0, 320.0, &mut bars, 0.15, false, mouse, false, &mut grab);
        assert_eq!(grab, 0);
        color_sliders_update(1600.0, 320.0, &mut bars, 0.0, true, mouse, true, &mut grab);
        assert_eq!(grab, 4);
        assert_eq!(bars.a, 0.5);
    }

    #[test]
    fn colour_sliders_leave_the_bars_alone_when_the_mouse_is_outside() {
        let mut bars = HslBars { h: 0.25, s: 0.5, l: 0.75, a: 0.5 };
        let before = bars;
        let outside = Vec2::new(10.0, 10.0);
        // Button down but over nothing: nothing moves and no grab starts.
        let mut grab: u8 = 0;
        let hl = color_sliders_update(
            1600.0, 320.0, &mut bars, 0.15, true, outside, true, &mut grab,
        );
        assert_eq!(bars, before);
        assert_eq!(grab, 0);
        assert!(!hl.iter().any(|h| *h));
        // Releasing the button clears a stale grab (`Main.cs:51112-51115`).
        let mut grab: u8 = 2;
        color_sliders_update(1600.0, 320.0, &mut bars, 0.15, true, outside, false, &mut grab);
        assert_eq!(grab, 0);
        assert_eq!(bars, before);
    }

    #[test]
    fn alpha_blips_use_the_xna_color_mul_truncation() {
        // `Main.cs:51242-51246`: `color6 * (num61/num54)`, where `color6` is
        // `hslToRgb(h, s, l)` with alpha 255. 84/167 = 0.502994..:
        // 200 -> 100.59, 100 -> 50.29, 50 -> 25.14, 255 -> 128.26, all truncated.
        // (The plan table's "127" alpha contradicts its own truncation rule.)
        assert_eq!(color::mul([200, 100, 50, 255], 84.0 / 167.0), [100, 50, 25, 128]);
    }
}
