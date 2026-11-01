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
        r.push_ui_sprite(
            [pen.x + g.pad[0] * scale, pen.y + g.pad[1] * scale],
            [g.w * scale, g.h * scale],
            g.uv,
            color,
        );
        pen.x += (g.kerning[1] + g.kerning[2]) * scale;
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

/// The Death-font width a large `UITextPanel` measures (`UITextPanel.cs:126-131`),
/// or the bitmap fallback's.
pub(crate) fn large_text_width(r: &SpriteRenderer, lookups: &assets::Lookups, text: &str) -> f32 {
    match &lookups.death_font {
        Some(f) => font_width(f, text, 1.0),
        None => r.text_width(text, 1.0),
    }
}
