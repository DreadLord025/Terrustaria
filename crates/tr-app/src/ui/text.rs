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
