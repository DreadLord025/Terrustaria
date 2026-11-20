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
use super::{in_rect, in_rect_strict};
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
/// The consumers loop it differently: the value bar runs
/// `for (num4 = 0f; num4 < (float)num; num++)` (`IngameOptions.cs:1552`) — 167
/// blips — while the HSL rows run `num <= num54` (`Main.cs:51169`, `:51206`,
/// `:51243`) — 168 blips.
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
/// (`IngameOptions.cs:1556`). The menu screens draw at scale 1
/// (`Main.cs:51535`); the in-game window draws at 0.75
/// (`IngameOptions.cs:116`'s `float scale = 0.75f`) — use
/// [`value_bar_rects_at`] for that one.
pub fn value_bar_rects(right_x: f32, center_y: f32) -> ([f32; 4], [f32; 4]) {
    value_bar_rects_at(right_x, center_y, 1.0)
}

/// [`value_bar_rects`] at the C#'s `scale` (`DrawValueBar`'s second argument):
/// `vector = (Width, Height) * scale` (`:1544`), the rect is
/// `(valuePosition.X - vector.X, valuePosition.Y - vector.Y / 2, vector.X,
/// vector.Y)` with the int truncations of `:1545-1546`, and the hit rect is
/// `Inflate((int)(-5f * scale), 0)` (`:1556`).
pub fn value_bar_rects_at(right_x: f32, center_y: f32, scale: f32) -> ([f32; 4], [f32; 4]) {
    let w = (BAR_W * scale) as i32 as f32;
    let h = (BAR_H * scale) as i32 as f32;
    let x = (right_x - w) as i32 as f32;
    let y = (center_y - h / 2.0) as i32 as f32;
    let rect = [x, y, w, h];
    let inflate = (5.0 * scale) as i32 as f32;
    let hit = [rect[0] + inflate, rect[1], rect[2] - inflate * 2.0, rect[3]];
    (rect, hit)
}

/// `IngameOptions.cs:1570-1579` over the *post-Inflate* rectangle: `:1556`
/// mutates the bar rect with `Inflate((int)(-5f*scale), 0)` (X + 5, W - 10)
/// before the read, so the caller passes the drawn (uninflated) `rect` and this
/// divides over the hit geometry — `(mouseX - rect.X) / rect.Width` while the
/// mouse is inside it (both edges inclusive per `Main.mouseX >= rectangle.X &&
/// Main.mouseX <= rectangle.X + rectangle.Width`), 0 left of it, 1 right of it.
/// The scale-1 menu form; the in-game window's 0.75 bars use
/// [`value_bar_value_at`].
pub fn value_bar_value(rect: [f32; 4], mouse_x: f32) -> f32 {
    value_bar_value_at(rect, mouse_x, 1.0)
}

/// [`value_bar_value`] at the bar's draw scale — the divisor is the inflated
/// hit rect (`IngameOptions.cs:1556`), not the drawn one.
pub fn value_bar_value_at(rect: [f32; 4], mouse_x: f32, scale: f32) -> f32 {
    // `rectangle.Inflate((int)(-5f * scale), 0)` (`IngameOptions.cs:1556`).
    let inflate = (5.0 * scale) as i32 as f32;
    let x = rect[0] + inflate;
    let w = rect[2] - inflate * 2.0;
    // The clamp reproduces the C# branch chain exactly: inside [x, x + w] the
    // ratio already lies in [0, 1]; left of x it is negative (C# returns 0f),
    // right of x + w it exceeds 1 (C# returns 1f).
    ((mouse_x - x) / w).clamp(0.0, 1.0)
}

/// Knob centre (`IngameOptions.cs:1566`, drawn with the sheet's centre as
/// origin): `(rect.x + 5 + 167*value, rect.y + 8)` at scale 1.
pub fn value_bar_knob(rect: [f32; 4], value: f32) -> [f32; 2] {
    value_bar_knob_at(rect, value, 1.0)
}

/// [`value_bar_knob`] at the draw scale: `num2 = rect.X + 5f*scale`,
/// `num3 = rect.Y + 4f*scale` (`:1550-1551`), the knob at
/// `(num2 + 167f*scale*perc, num3 + 4f*scale)` (`:1566`).
pub fn value_bar_knob_at(rect: [f32; 4], value: f32, scale: f32) -> [f32; 2] {
    let num2 = rect[0] + 5.0 * scale;
    let num3 = rect[1] + 4.0 * scale;
    [num2 + BLIPS as f32 * scale * value, num3 + 4.0 * scale]
}

/// Draws one value bar: the ColorBar sheet, `BLIPS` ColorBlips in
/// `color_lerp_black_to_white(i/167)` — the C# loop `for (num4 = 0f; num4 <
/// (float)num; num++)` with `num = 167` (`IngameOptions.cs:1549-1555`) draws 167
/// blips, not 168 — the `Slider_Highlight` overlay tinted `OurFavoriteColor`
/// while the mouse is inside the hit box (`IngameOptions.cs:1557-1564`), then
/// the ColorSlider knob. The menu screens' scale-1 form
/// (`Main.cs:51535` passes `1f`); the in-game window uses
/// [`draw_value_bar_at`] with its 0.75 scale.
// Drawn by the volume (26) and parallax (28) screens (`menu.rs`).
pub fn draw_value_bar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    right_x: f32,
    center_y: f32,
    value: f32,
    mouse: Vec2,
) {
    draw_value_bar_at(r, lookups, right_x, center_y, value, mouse, 1.0, color_lerp_black_to_white);
}

/// [`draw_value_bar`] at the C#'s `scale` with its `colorMethod`
/// (`DrawValueBar(sb, scale, perc, lockState, colorMethod)`,
/// `IngameOptions.cs:1537-1578`): the blips ride `colorMethod(num4 / 167)`
/// (`:1554`), the sheets scale with `scale`, and the knob's origin is the
/// scaled sheet centre. `lockState` is always 0 on the port's callers (the
/// highlight's `flag || lockState == 1` arm reduces to the hover test).
#[allow(clippy::too_many_arguments)]
pub fn draw_value_bar_at(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    right_x: f32,
    center_y: f32,
    value: f32,
    mouse: Vec2,
    scale: f32,
    color_method: impl Fn(f32) -> [u8; 4],
) {
    let (rect, hit) = value_bar_rects_at(right_x, center_y, scale);
    // `sb.Draw(value, rectangle, Color.White)` (`IngameOptions.cs:1548`).
    if let Some(uv) = lookups.hud_uv(lookups.ui_color_bar.as_ref()) {
        r.push_ui_sprite([rect[0], rect[1]], [rect[2], rect[3]], uv, [1.0; 4]);
    }
    // The gradient blips (`IngameOptions.cs:1550-1555`): one per step, starting
    // `5f*scale` in and `4f*scale` down, each `BLIP * scale` in size.
    if let Some(uv) = lookups.hud_uv(lookups.ui_color_blip.as_ref()) {
        for i in 0..BLIPS {
            let c = color_method(i as f32 / BLIPS as f32);
            r.push_ui_sprite(
                [rect[0] + 5.0 * scale + i as f32 * scale, rect[1] + 4.0 * scale],
                [BLIP_W * scale, BLIP_H * scale],
                uv,
                color::tint(c),
            );
        }
    }
    // `TextureAssets.ColorHighlight` over the whole bar while hovered
    // (`IngameOptions.cs:1557-1564`) — the inclusive-`Rectangle.Contains` test.
    if in_rect(mouse, hit) {
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
        let centre = value_bar_knob_at(rect, value, scale);
        r.push_ui_sprite(
            [centre[0] - KNOB_W * scale / 2.0, centre[1] - KNOB_H * scale / 2.0],
            [KNOB_W * scale, KNOB_H * scale],
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
/// `SpriteBatch.Draw` has a zero origin and the C# int-divides the sheet sizes
/// *before* the float cast — `ColorSlider.Width()/2 = 5`,
/// `ColorSlider.Height()/2 = 12`, `Hue.Height()/2 = 8` — so the offset is
/// `(left + HUE_TRAVEL*t - 5.0, top - 12.0 + 8.0)`, not the float halves.
pub fn color_slider_knob(left: f32, top: f32, t: f32) -> [f32; 2] {
    // `(float)(TextureAssets.ColorSlider.Width() / 2)` and friends
    // (`Main.cs:51146`): integer division (5, 12, 8), then the cast.
    [
        left + HUE_TRAVEL * t - (KNOB_W as i32 / 2) as f32,
        top - (KNOB_H as i32 / 2) as f32 + (BAR_H as i32 / 2) as f32,
    ]
}

/// Whether one colour-slider row takes the mouse: it strictly hovers the row's
/// hit box while no grab is held, or it holds the grab ((hover && grab == 0) ||
/// grab == id — `Main.cs:51142`, `:51177`, `:51212`, `:51253`). The strict
/// four-side comparison is `ui::in_rect_strict` (`Main.cs:51142`'s open-on-
/// every-edge test), unlike the value bar's inclusive `Rectangle.Contains`.
fn bar_active(left: f32, top: f32, id: u8, mouse: Vec2, grab: u8) -> bool {
    (in_rect_strict(mouse, color_slider_hit(left, top)) && grab == 0) || grab == id
}

/// The dragged 0..1 value of a row: `(mouseX - num53) / Hue.Width()`, clamped
/// (`Main.cs:51154-51160` and the row twins).
fn bar_value(left: f32, mouse_x: f32) -> f32 {
    ((mouse_x - left) / BAR_W).clamp(0.0, 1.0)
}

/// The draw-time drag half of the HSL block (`Main.cs:51102-51284`) as one pure
/// pass, so [`draw_color_sliders`] keeps hit-test → mutate in the same pass that
/// draws (Review Focus 1) and the grab semantics stay testable without a GPU.
/// Also drives the `menu.rs` colour-screen tests, which compose it with the
/// seeding and recompose steps around this function.
///
/// Releasing the button clears the sticky grab (`Main.cs:51112-51115`); while it
/// is down, an active row takes the grab (1 hue, 2 sat, 3 lum, 4 alpha) and
/// follows the mouse. The luminosity grab re-floors the stored value with
/// `lum_floor` — 0.15 for menuMode 25, 0.0 for 252 (`Main.cs:51201-51202`):
/// `l = t * (1 - floor) + floor` (`Main.cs:51232`). Returns the four
/// `Slider_Highlight` flags.
#[allow(clippy::too_many_arguments)]
pub(crate) fn color_sliders_update(
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
// Consumed by the cursor / border colour screens (`menu.rs`).
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

/// `(float)num / (float)num54` of the C# blip loops (`Main.cs:51171` and the row
/// twins). C# `hslToRgb` takes `float` parameters (`Main.cs:47890`), so the
/// ratio is computed in `f32` exactly as upstream and only widened to the `f64`
/// parameters of the Rust `color::hsl_to_rgb` port at the call sites.
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

/// `UISliderBase` (`GameContent/UI/Elements/UISliderBase.cs:5-36`),
/// research/18b row 52 (W4-d83): the shared lock/aim statics every slider
/// reads. The C# keeps two `static UIElement` pointers; the port keys them
/// by the caller's slider id (`Option<usize>`, `None` = null). The
/// `char_creation.rs` HSL bars already mirror the same pair for their three
/// bars (`CurrentLockedSlider` at `:172`, the `!Main.mouseLeft` release at
/// `:1183`); this is the element-level form the vertical sliders (row 57)
/// and any future slider share.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SliderLocks {
    /// `CurrentLockedSlider` (`UISliderBase.cs:13`).
    pub locked: Option<usize>,
    /// `CurrentAimedSlider` (`:15`).
    pub aimed: Option<usize>,
}

/// `GetUsageLevel`'s three states (`UISliderBase.cs:7-11`).
pub const USAGE_NOT_SELECTED: i32 = 0;
pub const USAGE_SELECTED_AND_LOCKED: i32 = 1;
pub const USAGE_OTHER_ELEMENT_IS_LOCKED: i32 = 2;

impl SliderLocks {
    /// `GetUsageLevel` (`:17-29`): this slider locked → 1; another locked →
    /// 2; none → 0.
    pub fn usage_level(&self, id: usize) -> i32 {
        if self.locked == Some(id) {
            USAGE_SELECTED_AND_LOCKED
        } else if self.locked.is_some() {
            USAGE_OTHER_ELEMENT_IS_LOCKED
        } else {
            USAGE_NOT_SELECTED
        }
    }

    /// `EscapeElements` (`:31-35`): both statics null — the Escape key's
    /// release (`UIColoredSlider`/`UIVerticalSlider` call it).
    pub fn escape_elements(&mut self) {
        self.locked = None;
        self.aimed = None;
    }
}

/// `UIVerticalSlider` (`GameContent/UI/Elements/UIVerticalSlider.cs:9-140`),
/// research/18b row 57 (W4-d83): the vertical volume-style slider — a
/// spliced `ColorBar` frame (5/4/4/4 depths) around a `MagicPixel` column
/// that fills bottom-up, with a white knob line at the fill's top edge.
/// The lock/aim machine rides [`SliderLocks`] (row 52); the two hover sound
/// latches (`_soundedUsage`, `_alreadyHovered`) are independent one-shots.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerticalSlider {
    /// `FillPercent` (`:11`) — refreshed from the getter every draw (`:47`).
    pub fill_percent: f32,
    /// `FilledColor = Main.OurFavoriteColor` (`:13`) — (255, 231, 69).
    pub filled_color: [u8; 4],
    /// `EmptyColor = Color.Black` (`:15`).
    pub empty_color: [u8; 4],
    /// `_isReallyMouseOvered` (`:23`) — the raw hit, refreshed by the bar
    /// pass (`:107`); the gamepad action reads it (`:75-78`).
    pub really_mouse_overed: bool,
    /// `_soundedUsage` (`:25`) — the keyboard-slide one-shot (`:60-68`).
    sounded_usage: bool,
    /// `_alreadyHovered` (`:27`) — the bar-hover one-shot (`:121-132`).
    already_hovered: bool,
}

impl Default for VerticalSlider {
    fn default() -> Self {
        Self {
            fill_percent: 0.0,
            filled_color: OUR_FAVORITE_COLOR,
            empty_color: [0, 0, 0, 255],
            really_mouse_overed: false,
            sounded_usage: false,
            already_hovered: false,
        }
    }
}

/// The bar's geometry (`UIVerticalSlider.cs:85-106`), all int rects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerticalBarRects {
    /// The spliced `ColorBar` frame — the element rect, depths
    /// left=5 right=4 top=4 bottom=4 (`:86-87`, the `Rectangle(5,4,4,4)`
    /// reads X/Width/Y/Height).
    pub frame: [i32; 4],
    /// The inner column (`:88-92`): X += 5, Width -= 9 (rectangle2.Right),
    /// Y += 4, Height -= 8 (rectangle2.Bottom) — the Right/Bottom are the
    /// *edges*, not the widths, so the insets are asymmetric.
    pub inner: [i32; 4],
    /// The filled part (`:96-99`): height = `(int)(inner.H * FillPercent)`,
    /// y pushed down by the empty remainder — the fill grows **upward**.
    pub filled: [i32; 4],
    /// The knob line (`:100-104`): `CenteredRectangle((filled.Center.X + 1,
    /// filled.Top), (filled.W + 16, 4))` (`Utils.cs:947-950` float form),
    /// then a Black copy inflated (2, 2) under the White one.
    pub knob: [i32; 4],
    /// The hit rect (`:105-106`): the inner column inflated (4, 0).
    pub hit: [i32; 4],
}

impl VerticalSlider {
    /// `DrawValueBarDynamicWidth`'s geometry (`:85-106`). `rect` is the
    /// element's `ToRectangle()` (already truncated).
    pub fn bar_rects(&self, rect: [i32; 4]) -> VerticalBarRects {
        let inner = [
            rect[0] + 5,          // `:89` rectangle2.Left = 5
            rect[1] + 4,          // `:91` rectangle2.Top = 4
            rect[2] - 9,          // `:90` rectangle2.Right = 5 + 4
            rect[3] - 8,          // `:92` rectangle2.Bottom = 4 + 4
        ];
        let fill_h = (inner[3] as f32 * self.fill_percent) as i32; // `:97`
        let filled = [inner[0], inner[1] + inner[3] - fill_h, inner[2], fill_h]; // `:98`
        // `:100` — `Rectangle.Center` is `X + Width / 2` in **integer**
        // division (XNA), then +1; CenteredRectangle's float form
        // (`Utils.cs:947-950`) truncates each part after the halving.
        let center_x = (filled[0] + filled[2] / 2 + 1) as f32;
        let knob_w = filled[2] + 16;
        let knob = [
            (center_x - knob_w as f32 / 2.0) as i32,
            (filled[1] as f32 - 4.0 / 2.0) as i32,
            knob_w,
            4,
        ];
        VerticalBarRects {
            frame: rect,
            inner,
            filled,
            knob,
            hit: [inner[0] - 4, inner[1], inner[2] + 8, inner[3]], // `:106` Inflate(4, 0)
        }
    }

    /// One `DrawSelf` pass (`:39-79` + `:107-138`). `id` is this slider's
    /// key in the shared [`SliderLocks`]; `mouse` the screen pointer;
    /// `mouse_left` = `Main.mouseLeft`, `mouse_left_trigger` =
    /// `PlayerInput.Triggers.Current.MouseLeft`, `ignore_mouse` =
    /// `PlayerInput.IgnoreMouseInterface`, `ignores_interaction` = the
    /// element's `IgnoresMouseInteraction`. Returns
    /// `(slide_value, hover_sound, usage_sound, gamepad_action)`:
    /// `slide_value` is the keyboard action's argument (`Some` fires it),
    /// the two bools are the one-shot `PlaySound(12)` requests, and
    /// `gamepad_action` fires `_slideGamepadAction` (`:75-78`).
    #[allow(clippy::too_many_arguments)]
    pub fn draw_frame(
        &mut self,
        id: usize,
        locks: &mut SliderLocks,
        rect: [i32; 4],
        mouse: [f32; 2],
        mouse_left: bool,
        mouse_left_trigger: bool,
        using_gamepad: bool,
        ignore_mouse: bool,
        ignores_interaction: bool,
        fill_value: f32,
    ) -> (Option<f32>, bool, bool, bool) {
        locks.aimed = None; // `:41`
        if !mouse_left || ignore_mouse {
            locks.locked = None; // `:42-45`
        }
        self.fill_percent = fill_value; // `:47`
        let rects = self.bar_rects(rect);
        // `:107` — the raw hit: Rectangle.Contains on the truncated point.
        let mx = mouse[0] as i32;
        let my = mouse[1] as i32;
        let h = rects.hit;
        self.really_mouse_overed =
            mx >= h[0] && mx < h[0] + h[2] && my >= h[1] && my < h[1] + h[3] && !ignore_mouse;
        let mut flag = self.really_mouse_overed;
        if ignores_interaction {
            flag = false; // `:108-111`
        }
        let usage = locks.usage_level(id);
        if usage == USAGE_OTHER_ELEMENT_IS_LOCKED {
            flag = false; // `:113-116`
        }
        if usage == USAGE_SELECTED_AND_LOCKED {
            flag = true; // `:117-120`
        }
        // The bar-hover one-shot (`:121-132`).
        let mut hover_sound = false;
        if flag || usage == USAGE_SELECTED_AND_LOCKED {
            if !self.already_hovered {
                hover_sound = true; // `:123-126`
            }
            self.already_hovered = true;
        } else {
            self.already_hovered = false;
        }
        // The value (`:133-137`): GetLerpValue(bottom, top, mouseY, clamped)
        // — bottom-up, so the top of the column reads 1.
        let mut slide_value = None;
        let bar_set = if flag {
            let from = (rects.inner[1] + rects.inner[3]) as f32; // Bottom
            let to = rects.inner[1] as f32; // Top
            let t = ((mouse[1] - from) / (to - from)).clamp(0.0, 1.0);
            slide_value = Some(t);
            true
        } else {
            false
        };
        // `:54-70` — the aim/lock + keyboard action with its own one-shot.
        let mut usage_sound = false;
        if locks.locked == Some(id) || bar_set {
            locks.aimed = Some(id); // `:56`
            if mouse_left_trigger && !using_gamepad && locks.locked == Some(id) {
                if !self.sounded_usage {
                    usage_sound = true; // `:60-63`
                }
                self.sounded_usage = true;
            } else {
                self.sounded_usage = false; // `:66-69`
                slide_value = None; // the action only fires in that arm (`:57-59`)
            }
        }
        if locks.aimed.is_some() && locks.locked.is_none() {
            locks.locked = locks.aimed; // `:71-74`
        }
        (
            slide_value,
            hover_sound,
            usage_sound,
            self.really_mouse_overed, // `:75-78`
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The bar geometry (`UIVerticalSlider.cs:85-106`): a 20×100 rect at
    /// (100, 50) insets to (105, 54, 11, 92) — the Right/Bottom edge reads
    /// make it -9/-8, not -8/-8; a 0.5 fill takes the bottom 46 px; the
    /// knob centres on the fill top +1 with the float truncation.
    #[test]
    fn the_vertical_bar_insets_by_edges_and_fills_upward() {
        let mut s = VerticalSlider::default();
        assert_eq!(s.filled_color, [255, 231, 69, 255]);
        s.fill_percent = 0.5;
        let r = s.bar_rects([100, 50, 20, 100]);
        assert_eq!(r.inner, [105, 54, 11, 92]);
        assert_eq!(r.filled, [105, 54 + 46, 11, 46], "the fill sits at the bottom");
        // Center.X = 105 + 11/2 = 110 (int halves) + 1 = 111; knob w = 27,
        // x = (int)(111 - 13.5) = 97; y = fillTop - 2 = 98.
        assert_eq!(r.knob, [97, 98, 27, 4]);
        assert_eq!(r.hit, [101, 54, 19, 92]);
        s.fill_percent = 1.0;
        assert_eq!(s.bar_rects([100, 50, 20, 100]).filled, [105, 54, 11, 92]);
        s.fill_percent = 0.0;
        assert_eq!(s.bar_rects([100, 50, 20, 100]).filled[3], 0);
    }

    /// The lock machine + one-shots (`UIVerticalSlider.cs:39-79`,
    /// `:107-138`): a fresh in-hit hover sounds once and grabs the lock;
    /// the locked slider fires the keyboard action (with its own one-shot)
    /// even outside the hit; the value reads bottom-up clamped.
    #[test]
    fn the_vertical_slider_grabs_locks_and_sounds_once() {
        let mut s = VerticalSlider::default();
        let mut locks = SliderLocks::default();
        let rect = [100, 50, 20, 100];
        // Hover mid-column (y=100), mouse held: hover sound, lock grab, no
        // keyboard action yet (the lock did not exist at the arm's check).
        let (v, hover, usage, gp) =
            s.draw_frame(0, &mut locks, rect, [110.0, 100.0], true, true, false, false, false, 0.25);
        assert!(hover, "the first hover sounds (`:123-126`)");
        assert!(!usage);
        assert!(v.is_none());
        assert!(gp);
        assert_eq!(locks.locked, Some(0));
        assert_eq!(s.fill_percent, 0.25, "the getter refresh (`:47`)");
        // Second frame, still held: no hover sound; the keyboard action
        // fires with its own one-shot. Value = (146-100)/(146-54) = 0.5.
        let (v, hover, usage, _) =
            s.draw_frame(0, &mut locks, rect, [110.0, 100.0], true, true, false, false, false, 0.25);
        assert!(!hover);
        assert!(usage, "the first sliding frame sounds (`:60-63`)");
        assert_eq!(v, Some(0.5));
        let (v, _, usage, _) =
            s.draw_frame(0, &mut locks, rect, [110.0, 100.0], true, true, false, false, false, 0.25);
        assert!(!usage, "the usage latch holds");
        assert_eq!(v, Some(0.5));
        // Locked but pointer far away: flag forced true by usage 1, the
        // value clamps to the top (1.0 above the column).
        let (v, _, _, gp) =
            s.draw_frame(0, &mut locks, rect, [500.0, 0.0], true, true, false, false, false, 0.25);
        assert_eq!(v, Some(1.0), "clamped GetLerpValue (`:135`)");
        assert!(!gp, "the raw hit is false out there");
        // Release: the lock drops, everything re-arms.
        let (v, _, _, _) =
            s.draw_frame(0, &mut locks, rect, [500.0, 0.0], false, false, false, false, false, 0.25);
        assert!(v.is_none());
        assert_eq!(locks.locked, None);
    }

    /// `GetUsageLevel`/`EscapeElements` (`UISliderBase.cs:17-35`): own lock
    /// → 1, other lock → 2, free → 0; escape clears both statics.
    #[test]
    fn the_slider_locks_report_the_three_usage_levels() {
        let mut locks = SliderLocks::default();
        assert_eq!(locks.usage_level(0), USAGE_NOT_SELECTED);
        locks.locked = Some(0);
        locks.aimed = Some(1);
        assert_eq!(locks.usage_level(0), USAGE_SELECTED_AND_LOCKED);
        assert_eq!(locks.usage_level(1), USAGE_OTHER_ELEMENT_IS_LOCKED);
        locks.escape_elements();
        assert_eq!(locks.locked, None);
        assert_eq!(locks.aimed, None);
        assert_eq!(locks.usage_level(0), USAGE_NOT_SELECTED);
    }

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
    fn value_bar_value_divides_over_the_inflated_hit_rect() {
        let (rect, _) = value_bar_rects(797.0, 332.0);
        // `IngameOptions.cs:1556` inflates the drawn rect to X = 624, W = 168
        // before `:1570-1579` divide, so the hit geometry is the divisor.
        // Centre: (708 - 624) / 168 = 0.5.
        assert_eq!(value_bar_value(rect, 708.0), 0.5, "708 = 624 + 84");
        // The inflated slope, not the drawn one: 666 is 42 px into the hit box
        // (0.25), not 47 px into the drawn bar ((666-619)/178 ~= 0.264).
        assert_eq!(value_bar_value(rect, 666.0), 0.25);
        // Both hit edges are inclusive (`Main.mouseX >= rectangle.X &&
        // Main.mouseX <= rectangle.X + rectangle.Width`).
        assert_eq!(value_bar_value(rect, 624.0), 0.0);
        assert_eq!(value_bar_value(rect, 792.0), 1.0);
        // Positional clamp outside the hit box (`IngameOptions.cs:1575-1579`):
        // left of X reads 0 (619 is the drawn left edge), right of X + W reads 1.
        assert_eq!(value_bar_value(rect, 619.0), 0.0);
        assert_eq!(value_bar_value(rect, 797.0), 1.0);
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
        // `Main.cs:51146`: the C# int-divides the sheet sizes before the float
        // cast (11/2 = 5, 25/2 = 12, 16/2 = 8), so
        // x = 711 + 176*0.5 - 5 = 794 and y = 320 - 12 + 8 = 316.
        assert_eq!(color_slider_knob(711.0, 320.0, 0.5), [794.0, 316.0]);
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
