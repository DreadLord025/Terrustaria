//! HUD text: the original `Mouse_Text`/`ItemStack` fonts when the retail assets are
//! present, the built-in bitmap font otherwise.

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;

/// `DynamicSpriteFont.MeasureString`.
pub(crate) fn font_width(font: &assets::UiFont, text: &str, scale: f32) -> f32 {
    let mut w = 0.0;
    let mut max_z = 0.0f32;
    let mut first = true;
    for c in text.chars() {
        if c == '\n' {
            break;
        }
        if c == '\r' {
            continue;
        }
        let Some(g) = font.glyph(c) else { continue };
        let mut bearing = g.kerning[0];
        if first {
            bearing = bearing.max(0.0);
        } else {
            w += (font.spacing + max_z) * scale;
        }
        w += (bearing + g.kerning[1]) * scale;
        max_z = g.kerning[2];
        first = false;
    }
    w + max_z.max(0.0) * scale
}

/// `DynamicSpriteFont.DrawString`: the pen advances by `spacing + kerning.X`, the
/// glyph is drawn at `pen + padding`, then the pen advances by `kerning.Y + kerning.Z`.
pub(crate) fn push_font_text(
    r: &mut SpriteRenderer,
    font: &assets::UiFont,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    let mut pen = Vec2::new(x, y);
    let mut first = true;
    for c in text.chars() {
        match c {
            '\n' => {
                pen.x = x;
                pen.y += font.line_spacing * scale;
                first = true;
                continue;
            }
            '\r' => continue,
            _ => {}
        }
        let Some(g) = font.glyph(c) else { continue };
        let mut bearing = g.kerning[0];
        if first {
            bearing = bearing.max(0.0);
        } else {
            pen.x += font.spacing * scale;
        }
        pen.x += bearing * scale;
        r.push_font_glyph(
            [pen.x + g.pad[0] * scale, pen.y + g.pad[1] * scale],
            [g.w * scale, g.h * scale],
            g.uv,
            color,
            g.layer,
        );
        pen.x += (g.kerning[1] + g.kerning[2]) * scale;
        first = false;
    }
}

/// `DynamicSpriteFont.DrawString` with a `Vector2` scale — the achievement
/// description's `vector3 = (0.92, clampedY)` (`UIAchievementListItem.cs:101`,
/// `:109-112`, `:130`): X advances at `scale.X`, the line step and glyph
/// height at `scale.Y`. The uniform [`push_font_text`] is the `scale.X ==
/// scale.Y` special case.
#[allow(clippy::too_many_arguments)]
pub(super) fn push_font_text_xy(
    r: &mut SpriteRenderer,
    font: &assets::UiFont,
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    color: [f32; 4],
    text: &str,
) {
    let mut pen = Vec2::new(x, y);
    let mut first = true;
    for c in text.chars() {
        match c {
            '\n' => {
                pen.x = x;
                pen.y += font.line_spacing * scale_y;
                first = true;
                continue;
            }
            '\r' => continue,
            _ => {}
        }
        let Some(g) = font.glyph(c) else { continue };
        let mut bearing = g.kerning[0];
        if first {
            bearing = bearing.max(0.0);
        } else {
            pen.x += font.spacing * scale_x;
        }
        pen.x += bearing * scale_x;
        r.push_font_glyph(
            [pen.x + g.pad[0] * scale_x, pen.y + g.pad[1] * scale_y],
            [g.w * scale_x, g.h * scale_y],
            g.uv,
            color,
            g.layer,
        );
        pen.x += (g.kerning[1] + g.kerning[2]) * scale_x;
        first = false;
    }
}

/// UI label with the original `Mouse_Text` font, or the built-in bitmap font when
/// the retail fonts are unavailable (PNG-only fallback).
pub(crate) fn hud_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    match &lookups.ui_font {
        Some(f) => push_font_text(r, f, x, y, scale, color, text),
        None => r.push_text(x, y, scale, color, text),
    }
}

pub(super) fn hud_item_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    match &lookups.item_font {
        Some(f) => push_font_text(r, f, x, y, scale, color, text),
        None => r.push_text(x, y, scale, color, text),
    }
}

/// One `MouseText` line height (`DynamicSpriteFont.MeasureString(text).Y`): the
/// retail sheet's `line_spacing`, or the built-in bitmap fallback's 8 px.
pub(crate) fn line_height(lookups: &assets::Lookups) -> f32 {
    match &lookups.ui_font {
        Some(f) => f.line_spacing,
        None => 8.0,
    }
}

pub(crate) fn hud_text_width(r: &SpriteRenderer, lookups: &assets::Lookups, text: &str, scale: f32) -> f32 {
    match &lookups.ui_font {
        Some(f) => font_width(f, text, scale),
        None => r.text_width(text, scale),
    }
}

pub(super) fn hud_item_text_width(
    r: &SpriteRenderer,
    lookups: &assets::Lookups,
    text: &str,
    scale: f32,
) -> f32 {
    match &lookups.item_font {
        Some(f) => font_width(f, text, scale),
        None => r.text_width(text, scale),
    }
}

// --- The `Utils.DrawBorderString` family ----------------------------------------

/// Which game font a bordered string draws with. All three fall back to the
/// built-in bitmap font (`SpriteRenderer::push_text`) when the retail sheet is
/// missing (PNG-only runs).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum BorderFont {
    /// `Mouse_Text` (`Lookups::ui_font`) — `Utils.DrawBorderString`'s font.
    Ui,
    /// `Item_Stack` (`Lookups::item_font`) — `Utils.DrawBorderStringFourWay`'s
    /// font.
    Item,
    /// `Death` (`Lookups::death_font`) — `Utils.DrawBorderStringBig`'s font, the
    /// `UITextPanel` label font.
    Death,
}

/// `Color.Black` — every border pass of the port's bordered strings
/// (`Utils.cs:2468`, `:2524-2533`; the ChatManager shadow, `ChatManager.cs:
/// 200-205`).
const BORDER_BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// One pass of a bordered string through [`BorderFont`]'s font.
fn push_border_pass(
    font: BorderFont,
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    let sheet = match font {
        BorderFont::Ui => &lookups.ui_font,
        BorderFont::Item => &lookups.item_font,
        BorderFont::Death => &lookups.death_font,
    };
    match sheet {
        Some(f) => push_font_text(r, f, x, y, scale, color, text),
        None => r.push_text(x, y, scale, color, text),
    }
}

/// `Utils.DrawBorderString*`'s shared shape (`Utils.cs:2464-2535`): the `ring`'s
/// black border passes around `(x, y)`, then the colour pass on top. The ring
/// picks the style — the single (+1, +1) corner shadow of `DrawBorderString`,
/// the four ±2 px side passes of `DrawBorderStringFourWay`, the 3x3 ring of
/// `DrawBorderStringBig` — and `ChatManager`'s shadow is the same shape at its
/// own spread (`ChatManager.cs:200-205`).
pub(crate) fn draw_border_string(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    font: BorderFont,
    x: f32,
    y: f32,
    scale: f32,
    ring: &[[f32; 2]],
    color: [f32; 4],
    text: &str,
) {
    for [dx, dy] in ring {
        push_border_pass(font, r, lookups, x + dx, y + dy, scale, BORDER_BLACK, text);
    }
    push_border_pass(font, r, lookups, x, y, scale, color, text);
}

/// `Utils.DrawBorderString` (`Utils.cs:2510-2518`) at scale 1 through the ui
/// font: the port's 1 px black corner-shadow stand-in (the C# shadow is
/// `ChatManager.DrawColorCodedStringWithShadow`'s, `ChatManager.cs:200-205`),
/// then the colour — the same convention as `uistate::draw_text_panel`.
pub(crate) fn border_string(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    tint: [f32; 4],
    text: &str,
) {
    draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        x,
        y,
        1.0,
        &[[1.0, 1.0]],
        tint,
        text,
    );
}

/// `FontAssets.MouseText.MeasureString(text).X` at `scale` when the retail sheet
/// is loaded (the renderer-free twin of [`hud_text_width`]).
pub(crate) fn ui_font_width(lookups: &assets::Lookups, text: &str, scale: f32) -> Option<f32> {
    lookups.ui_font.as_ref().map(|f| font_width(f, text, scale))
}

/// `FontAssets.MouseText.MeasureString(text).Y` at `scale` — the retail
/// sheet's line spacing (the bitmap fallback has no metrics; the callers
/// estimate). `ChatManager.GetStringSize`'s Y (`ChatManager.cs:189-198`),
/// which `NPCChatPanel.DrawButtons`' label origin reads (`NPCChatPanel.cs:120`).
pub(crate) fn ui_font_height(lookups: &assets::Lookups, scale: f32) -> Option<f32> {
    lookups.ui_font.as_ref().map(|f| f.line_spacing * scale)
}

/// `FontAssets.DeathText.MeasureString(text).X` (`Main.DrawSettingButton`'s
/// `vector3.X`, `Main.cs:45536`) when the retail sheet is loaded.
pub(crate) fn death_font_width(lookups: &assets::Lookups, text: &str) -> Option<f32> {
    lookups.death_font.as_ref().map(|f| font_width(f, text, 1.0))
}

/// The Death-font width a large `UITextPanel` measures (`UITextPanel.cs:126-131`),
/// or the bitmap fallback's.
pub(crate) fn large_text_width(r: &SpriteRenderer, lookups: &assets::Lookups, text: &str) -> f32 {
    match &lookups.death_font {
        Some(f) => font_width(f, text, 1.0),
        None => r.text_width(text, 1.0),
    }
}

/// The death screen's centered line through the Death font —
/// `Main.DrawDeathText`'s `DrawString(DeathText, ...)` shape
/// (`Main.cs:45330-45341`): the text centered at the screen's midpoint X at
/// the given Y, the death-tint alpha applied by the caller.
pub(crate) fn draw_death_string(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen_x: f32,
    y: f32,
    color: [f32; 4],
    text: &str,
) {
    let w = match &lookups.death_font {
        Some(f) => font_width(f, text, 1.0),
        None => r.text_width(text, 1.0),
    };
    let x = screen_x / 2.0 - w / 2.0;
    draw_border_string(
        r,
        lookups,
        BorderFont::Death,
        x,
        y,
        1.0,
        &[[1.0, 1.0]],
        color,
        text,
    );
}

/// The death font's line height (`FontAssets.DeathText.MeasureString(...).Y`)
/// — the unscaled box the skip prompt's baseline math reads
/// (`Main.cs:45366`).
pub(crate) fn death_line_height(lookups: &assets::Lookups) -> f32 {
    lookups.death_font.as_ref().map_or(26.0, |f| f.line_spacing)
}

/// The death screen's scaled line — `ChatManager.DrawColorCodedString` at the
/// skip prompt's `(1 + num5) * num2` scale (`Main.cs:45363-45366`), centred
/// like [`draw_death_string`].
pub(crate) fn draw_death_string_scaled(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen_x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    let w = match &lookups.death_font {
        Some(f) => font_width(f, text, scale),
        None => r.text_width(text, scale),
    };
    let x = screen_x / 2.0 - w / 2.0;
    draw_border_string(
        r,
        lookups,
        BorderFont::Death,
        x,
        y,
        scale,
        &[[1.0, 1.0]],
        color,
        text,
    );
}

/// The death screen's small "Dropped coins" line — the `MouseText` font
/// (`Main.cs:45336-45338`), centered like the big line above.
pub(crate) fn draw_dropped_coins(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen_x: f32,
    y: f32,
    color: [f32; 4],
    text: &str,
) {
    let w = hud_text_width(r, lookups, text, 1.0);
    let x = screen_x / 2.0 - w / 2.0;
    border_string(r, lookups, x, y, color, text);
}

// --- UIText (the element half, W4-d83 research/18b row 53) --------------------
//
// `GameContent/UI/Elements/UIText.cs:13-195`. The immediate draws above
// already mirror its draw offsets (`crafting.rs:93-95`, `keyboard.rs:809`);
// this is the element's *state machine*: the InternalSetText metrics, the
// min-size pins and the DrawSelf position/scale maths as pure code. The
// snippet layout (`ChatManager.LayoutSnippets`) is the caller's — the
// measured width comes in, the wrap itself is `bestiary.rs::wrap_text`'s
// greedy pass.

/// `UIText` (`UIText.cs:13-195`), research/18b row 53 (W4-d83): the label
/// element every screen appends. The port keeps the fields the draw and the
/// layout read; the parsed snippet list is the caller's (the port's screens
/// draw plain strings).
#[derive(Clone, Debug, PartialEq)]
pub struct TextElement {
    /// `_text.ToString()` (`:15`, `:35`).
    pub text: String,
    /// `_textScale = 1f` (`:17`).
    pub text_scale: f32,
    /// `_isLarge` (`:21`) — DeathText font when true, MouseText otherwise
    /// (`:157`, `:178`).
    pub large: bool,
    /// `_color = Color.White` (`:23`) / `TextColor` (`:56-66`).
    pub color: [u8; 4],
    /// `_shadowColor = Color.Black` (`:25`) / `ShadowColor` (`:68-78`).
    pub shadow_color: [u8; 4],
    /// `_isWrapped` (`:27`) — the setter re-runs InternalSetText (`:49-53`).
    pub is_wrapped: bool,
    /// `DynamicallyScaleDownToWidth` (`:29`).
    pub dynamically_scale_down_to_width: bool,
    /// `TextOriginX = 0.5f` (`:37`, ctor `:84`).
    pub text_origin_x: f32,
    /// `TextOriginY = 0f` (`:39`, ctor `:85`).
    pub text_origin_y: f32,
    /// `WrappedTextBottomPadding = 20f` (`:41`, ctor `:87`).
    pub wrapped_text_bottom_padding: f32,
    /// `_textSize` (`:19`) — the InternalSetText result.
    pub text_size: [f32; 2],
    /// `MinWidth`/`MinHeight` pixels (`:188-189`).
    pub min_size: [f32; 2],
}

impl TextElement {
    /// The ctor (`:82-89`): origins (0.5, 0), wrap off, bottom padding 20,
    /// then `InternalSetText`. `measured_w` is the caller's
    /// `ChatManager.GetStringSize` width at `text_scale` (the wrapped case
    /// passes the laid-out block's size).
    pub fn new(text: &str, text_scale: f32, large: bool, measured: [f32; 2]) -> Self {
        let mut t = Self {
            text: String::new(),
            text_scale,
            large,
            color: [255, 255, 255, 255],
            shadow_color: [0, 0, 0, 255],
            is_wrapped: false,
            dynamically_scale_down_to_width: false,
            text_origin_x: 0.5,
            text_origin_y: 0.0,
            wrapped_text_bottom_padding: 20.0,
            text_size: [0.0, 0.0],
            min_size: [0.0, 0.0],
        };
        t.internal_set_text(text, text_scale, large, measured, [0.0; 4]);
        t
    }

    /// `InternalSetText`'s metrics half (`:170-194`): `_textSize` keeps the
    /// measured width; the height is the wrapped block's height **+
    /// WrappedTextBottomPadding * scale** (`:180-183`) or the fixed
    /// `(large ? 32 : 16) * scale` (`:186`). `MinWidth` is 0 when wrapped or
    /// dynamically scaled, else `_textSize.X + padding L+R`; `MinHeight` is
    /// always `_textSize.Y + padding T+B` (`:188-189`). `padding` is
    /// `[left, right, top, bottom]`.
    pub fn internal_set_text(
        &mut self,
        text: &str,
        text_scale: f32,
        large: bool,
        measured: [f32; 2],
        padding: [f32; 4],
    ) {
        self.text = text.to_string();
        self.text_scale = text_scale;
        self.large = large;
        let mut size = measured;
        if self.is_wrapped {
            size[1] += self.wrapped_text_bottom_padding * text_scale; // `:182`
        } else {
            size[1] = (if large { 32.0 } else { 16.0 }) * text_scale; // `:186`
        }
        self.text_size = size;
        self.min_size = [
            if self.is_wrapped || self.dynamically_scale_down_to_width {
                0.0
            } else {
                size[0] + padding[0] + padding[1] // `:188`
            },
            size[1] + padding[2] + padding[3], // `:189`
        ];
    }

    /// `SetText(text)` (`:106-109`): keeps the current scale/large; the
    /// caller re-measures.
    pub fn set_text(&mut self, text: &str, measured: [f32; 2], padding: [f32; 4]) {
        self.internal_set_text(text, self.text_scale, self.large, measured, padding);
    }

    /// `DrawSelf`'s geometry (`:126-160`). `inner` is
    /// `GetInnerDimensions()` `[x, y, w, h]`. Returns the draw position, the
    /// effective scale and the shadow tint:
    /// - y starts at `inner.y - (large ? 10 : 2) * scale` (`:132-139`);
    /// - `DynamicallyScaleDownToWidth` with `_textSize.X > inner.W` shrinks
    ///   scale and textSize by `inner.W / _textSize.X` (`:143-153`);
    /// - the origins centre what's left: `x += (W - textSize.X) * OriginX`,
    ///   `y += (H - textSize.Y) * OriginY` (`:154-155`);
    /// - the shadow is `_shadowColor * ((float)(int)_color.A / 255f)`
    ///   (`:156`) — the text's alpha fades the shadow with it.
    pub fn draw_geometry(&self, inner: [f32; 4]) -> ([f32; 2], f32, [u8; 4]) {
        let mut scale = self.text_scale;
        let mut size = self.text_size;
        let mut pos = [
            inner[0],
            inner[1] - (if self.large { 10.0 } else { 2.0 }) * self.text_scale,
        ];
        if self.dynamically_scale_down_to_width && size[0] > inner[2] {
            let num = inner[2] / size[0]; // `:145`
            scale *= num; // `:151`
            size = [size[0] * num, size[1] * num]; // `:152`
        }
        pos[0] += (inner[2] - size[0]) * self.text_origin_x; // `:154`
        pos[1] += (inner[3] - size[1]) * self.text_origin_y; // `:155`
        let shadow = super::color::mul(self.shadow_color, self.color[3] as f32 / 255.0);
        (pos, scale, shadow)
    }
}

#[cfg(test)]
mod element_tests {
    use super::*;

    /// `InternalSetText` (`UIText.cs:180-189`): unwrapped height is the
    /// fixed (16/32)*scale, wrapped adds 20*scale to the measured block;
    /// MinWidth zeroes for wrapped/dynamic, MinHeight always adds padding.
    #[test]
    fn the_text_metrics_pin_the_min_sizes() {
        let t = TextElement::new("Hi", 1.0, false, [20.0, 99.0]);
        assert_eq!(t.text_size, [20.0, 16.0], "the measured height is replaced");
        assert_eq!(t.min_size, [20.0, 16.0]);
        let big = TextElement::new("Hi", 0.8, true, [20.0, 99.0]);
        assert_eq!(big.text_size, [20.0, 25.6]); // 32 * 0.8
        let mut w = TextElement::new("a b c", 1.0, false, [40.0, 32.0]);
        w.is_wrapped = true;
        w.internal_set_text("a b c", 1.0, false, [40.0, 32.0], [2.0, 2.0, 1.0, 1.0]);
        assert_eq!(w.text_size, [40.0, 52.0], "32 + 20*1 (`:182`)");
        assert_eq!(w.min_size, [0.0, 54.0], "wrapped MinWidth 0 (`:188`)");
        let mut d = TextElement::new("long", 1.0, false, [80.0, 16.0]);
        d.dynamically_scale_down_to_width = true;
        d.internal_set_text("long", 1.0, false, [80.0, 16.0], [0.0; 4]);
        assert_eq!(d.min_size, [0.0, 16.0], "dynamic MinWidth 0 too");
    }

    /// `DrawSelf` (`UIText.cs:130-156`): the -2*scale shift, the 0.5
    /// horizontal centre, the dynamic shrink when the text overflows, and
    /// the shadow riding the text alpha (Black * 128/255 truncates to
    /// alpha 128).
    #[test]
    fn the_draw_geometry_centres_shrinks_and_fades_the_shadow() {
        let t = TextElement::new("Hi", 1.0, false, [20.0, 16.0]);
        let (pos, scale, shadow) = t.draw_geometry([100.0, 50.0, 60.0, 16.0]);
        // y = 50 - 2; x = 100 + (60-20)*0.5 = 120; OriginY 0 adds nothing.
        assert_eq!(pos, [120.0, 48.0]);
        assert_eq!(scale, 1.0);
        assert_eq!(shadow, [0, 0, 0, 255]);
        let mut d = TextElement::new("long", 1.0, false, [80.0, 16.0]);
        d.dynamically_scale_down_to_width = true;
        d.color = [255, 255, 255, 128];
        let (pos, scale, shadow) = d.draw_geometry([0.0, 0.0, 40.0, 16.0]);
        assert_eq!(scale, 0.5, "40/80 shrink (`:145-151`)");
        assert_eq!(pos, [0.0, -2.0], "the shrunk size fills the width");
        assert_eq!(shadow, [0, 0, 0, 128], "the alpha fades the shadow (`:156`)");
        // The large font shifts -10*scale instead (`:134`).
        let big = TextElement::new("Hi", 1.0, true, [20.0, 32.0]);
        assert_eq!(big.draw_geometry([0.0, 20.0, 20.0, 32.0]).0, [0.0, 10.0]);
    }
}
