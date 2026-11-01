//! The shared vanilla-UI kit behind the P6 front-end (`Main.menuMode == 888`):
//! the `UIElement` layout pass, the vanilla UI colour constants,
//! `Utils.DrawSplicedPanel` (with its `Utils.cs:2593` top-right quirk),
//! `UITextPanel`, `UIImageButton` and `UIColoredImageButton`.
//!
//! Screen-model ruling (plan §"Screen-model ruling"): this is an **immediate**
//! port — no retained `UIElement` tree, no `UserInterface`. What the P6 screens
//! share is the `UIElement.cs:306-322` layout pass, ported once as pure code
//! (`resolve`), plus these draw helpers, the same shape as `ui/crafting.rs` /
//! `ui/panel.rs`. Every draw takes its sheets as `Option`s, so
//! `Lookups::default()` (every sheet `None`) draws nothing and panics nowhere
//! (Review Focus 7) — exactly like `ui/panel.rs`.

use tr_render::{SheetRect, SpriteRenderer};

use super::panel;
use super::text::push_font_text;
use crate::assets;

/// `StyleDimension` (`UIElement.cs` `StyleDimension`): `pixels + percent * parent`.
///
/// `GetValue(containerSize)` = `Pixels + Precent * containerSize`
/// (`StyleDimension.cs:17-20`) — the percent is a **fraction** (0.5 = 50%),
/// exactly like the C# field.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Style {
    pub px: f32,
    pub pct: f32,
}

impl Style {
    /// `StyleDimension.FromPixelsAndPercent(px, pct)` (`StyleDimension.cs:44-46`).
    #[allow(dead_code)] // Consumed by Tasks 3-8 (the negative-offset setters).
    pub const fn new(px: f32, pct: f32) -> Self {
        Self { px, pct }
    }

    /// `StyleDimension.FromPixels` (`StyleDimension.cs:24-27`).
    #[allow(dead_code)] // Consumed by Tasks 3-8 (the pixel setters).
    pub const fn px(px: f32) -> Self {
        Self { px, pct: 0.0 }
    }

    /// `StyleDimension.FromPercent` (`StyleDimension.cs:29-32`).
    #[allow(dead_code)] // Consumed by Tasks 3-8 (the percent-based setters).
    pub const fn pct(pct: f32) -> Self {
        Self { px: 0.0, pct }
    }
}

/// XNA `MathHelper.Clamp`: `Math.Max(min, Math.Min(max, value))`. Unlike Rust's
/// `f32::clamp` this never panics — with `min > max` the min wins, as in XNA.
fn xna_clamp(value: f32, min: f32, max: f32) -> f32 {
    value.min(max).max(min)
}

/// The resolved box of one element — `UIElement.GetDimensionsBasedOnParentDimensions`
/// (`UIElement.cs:306-323`) with the margins left out (every P6 element leaves
/// them 0; `UIElement.cs:316-317` adds them back and `Recalculate` :293-297
/// subtracts them again, so they cancel at 0):
///
/// ```text
/// X = Left.GetValue(parentW) + parentX          (`UIElement.cs:308`)
/// Y = Top.GetValue(parentH) + parentY           (`UIElement.cs:309`)
/// W = Clamp(Width.GetValue(parentW), MinWidth, MaxWidth)   (`UIElement.cs:315`)
/// H = Clamp(Height.GetValue(parentH), MinHeight, MaxHeight) (`UIElement.cs:316`)
/// X += parentW*HAlign - W*HAlign                (`UIElement.cs:319`)
/// Y += parentH*VAlign - H*VAlign                (`UIElement.cs:320`)
/// ```
///
/// `parent` is the parent's INNER rect (`Recalculate`, `UIElement.cs:287-288`);
/// `min_w/min_h/max_w` are the **resolved** pixel values — `MinWidth`/`MinHeight`
/// are `StyleDimension.Empty` (0) and `MaxWidth` is `StyleDimension.Fill`
/// (`= parent width`, `UIElement.cs:33-39`) by default, and every P6 setter
/// overrides them with pixels-only dimensions (0%); pass `parent[2]` as `max_w`
/// for the default `Fill`. The `UIList` parent special case
/// (`parentDimensions.Height = float.MaxValue`, `UIElement.cs:289-291`) is not
/// modelled — the card rows derive from the list rect arithmetically (Task 3).
///
/// `padding` is this element's `(left, top, right, bottom)`. It does **not**
/// affect the outer rect — padding only shrinks what the element's own children
/// resolve against (`UIElement.cs:298-303`), which `inner` below applies.
/// The returned rect is the element's OUTER rect (== `GetDimensions()` at 0
/// margins, `UIElement.cs:385-388`).
#[allow(dead_code)] // Consumed by Tasks 3-8 (every screen's layout()).
#[allow(clippy::too_many_arguments)]
pub fn resolve(
    parent: [f32; 4],
    left: Style,
    top: Style,
    width: Style,
    height: Style,
    h_align: f32,
    v_align: f32,
    min_w: f32,
    min_h: f32,
    max_w: f32,
    padding: [f32; 4],
) -> [f32; 4] {
    // `UIElement.cs:308-309`.
    let mut x = left.px + left.pct * parent[2] + parent[0];
    let mut y = top.px + top.pct * parent[3] + parent[1];
    // `UIElement.cs:310-316`: the min/max clamp happens BEFORE the alignment.
    let w = xna_clamp(width.px + width.pct * parent[2], min_w, max_w);
    let h = xna_clamp(height.px + height.pct * parent[3], min_h, f32::MAX);
    // `UIElement.cs:319-320`.
    x += parent[2] * h_align - w * h_align;
    y += parent[3] * v_align - h * v_align;
    let _ = padding; // padding affects only `inner` (`UIElement.cs:298-303`), not the outer rect.
    [x, y, w, h]
}

/// The element's inner rect = outer − padding (`UIElement.GetInnerDimensions`,
/// the `Recalculate` tail `UIElement.cs:298-303`).
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub fn inner(rect: [f32; 4], padding: [f32; 4]) -> [f32; 4] {
    [
        rect[0] + padding[0],
        rect[1] + padding[1],
        rect[2] - padding[0] - padding[2],
        rect[3] - padding[1] - padding[3],
    ]
}

// --- The vanilla UI colours the P6 screens share (bytes, pre-`tint`) ---------

/// `Main.OurFavoriteColor = new Color(255, 231, 69)` (`Main.cs:66748`).
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub const OUR_FAVORITE_COLOR: [u8; 4] = [255, 231, 69, 255];

/// `Colors.FancyUIFatButtonMouseOver = Main.OurFavoriteColor` (`ID/Colors.cs:157`).
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub const FANCY_UI_FAT_BUTTON_MOUSE_OVER: [u8; 4] = OUR_FAVORITE_COLOR;

/// `Colors.InventoryDefaultColor = new Color(63, 65, 151, 255)` (`ID/Colors.cs:158`).
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub const INVENTORY_DEFAULT_COLOR: [u8; 4] = [63, 65, 151, 255];

/// `Main.mcColor = new Color(1f, 0.6f, 0f)` (`Main.cs:66765`) — the mediumcore
/// card colour; `(byte)(0.6f * 255) = 153`.
#[allow(dead_code)] // Consumed by Tasks 3-4.
pub const MC_COLOR: [u8; 4] = [255, 153, 0, 255];

/// `Main.hcColor = new Color(1f, 0.15f, 0.1f)` (`Main.cs:66766`) — the hardcore
/// card colour; XNA `(byte)` truncation: `(byte)(0.15f*255) = 38`, `(byte)(0.1f*255) = 25`.
#[allow(dead_code)] // Consumed by Tasks 3-4.
pub const HC_COLOR: [u8; 4] = [255, 38, 25, 255];

/// `Main.creativeModeColor = Color.Lerp(Color.HotPink, Color.White, 0.1f)`
/// (`Main.cs:66767`) — the Journey card colour; `Lerp` truncates per channel:
/// `(105 + (255-105)*0.1f) = 120`, `(180 + (255-180)*0.1f) = 187.5 -> 187`.
#[allow(dead_code)] // Consumed by Tasks 3-4.
pub const CREATIVE_MODE_COLOR: [u8; 4] = [255, 120, 187, 255];

/// `Main.legendaryModeColor = Color.LimeGreen` (`Main.cs:66768`).
#[allow(dead_code)] // Consumed by Tasks 3-4.
pub const LEGENDARY_MODE_COLOR: [u8; 4] = [50, 205, 50, 255];

/// `Color.Lerp(White, (63,65,151), 0.85) * 0.9` — the creation-screen
/// separators (`UICharacterCreation.cs:448,777`, `UIWorldCreation.cs:309`).
/// XNA truncates to bytes after each step: `Lerp -> (91, 93, 166, 255)`,
/// `*0.9 -> (81, 83, 149, 229)`.
#[allow(dead_code)] // Consumed by Tasks 7-8.
pub const CREATION_SEPARATOR_COLOR: [u8; 4] = [81, 83, 149, 229];

// --- Utils.DrawSplicedPanel --------------------------------------------------

/// `Utils.DrawSplicedPanel` (`Utils.cs:2581-2600`): the 9-piece table for a
/// `(x, y, w, h)` box and `(left, right, top, bottom)` slice depths. Returns
/// `(dest, src)` — the source rects are on a `sheet_w x sheet_h` sheet, exactly
/// the `texture.Width`/`texture.Height` the original reads (`Utils.cs:2592-2600`).
///
/// Ported **including** the `:2593` quirk (Review Focus 5): the third draw (the
/// top-right corner) takes its destination size as `(topEnd, rightEnd)` — width
/// `topEnd`, height `rightEnd` — while every other corner uses its own ends.
/// Shape-neutral at the 10/10/10/10 calls P6 makes, reproduced literally so an
/// asymmetric call stays faithful (don't "fix" the source, P4 Uncertainty-9
/// precedent). The box is clamped to at least `left+right` x `top+bottom`
/// first (`Utils.cs:2582-2587`).
#[allow(dead_code)] // Consumed by Tasks 3-4 (the card DrawPanel calls).
pub fn spliced_rects(
    rect: [f32; 4],
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    sheet_w: f32,
    sheet_h: f32,
) -> ([[f32; 4]; 9], [[f32; 4]; 9]) {
    let [x, y, w0, h0] = rect;
    // `if (w < leftEnd + rightEnd) w = leftEnd + rightEnd;` (`Utils.cs:2582-2584`).
    let w = w0.max(left + right);
    // `if (h < topEnd + bottomEnd) h = topEnd + bottomEnd;` (`Utils.cs:2585-2587`).
    let h = h0.max(top + bottom);
    let mid_w = w - left - right;
    let mid_h = h - top - bottom;
    let smid_w = sheet_w - left - right;
    let smid_h = sheet_h - top - bottom;
    let dst = [
        // `new Rectangle(x, y, leftEnd, topEnd)` (`:2591`).
        [x, y, left, top],
        // `new Rectangle(x + leftEnd, y, w - leftEnd - rightEnd, topEnd)` (`:2592`).
        [x + left, y, mid_w, top],
        // `Utils.cs:2593` — the quirk: destination `(x + w - rightEnd, y, topEnd,
        // rightEnd)` — `topEnd` WIDE, not `rightEnd`.
        [x + w - right, y, top, right],
        // `new Rectangle(x, y + topEnd, leftEnd, h - topEnd - bottomEnd)` (`:2594`).
        [x, y + top, left, mid_h],
        // `new Rectangle(x + leftEnd, y + topEnd, w - ..., h - ...)` (`:2595`).
        [x + left, y + top, mid_w, mid_h],
        // `new Rectangle(x + w - rightEnd, y + topEnd, rightEnd, h - ...)` (`:2596`).
        [x + w - right, y + top, right, mid_h],
        // `new Rectangle(x, y + h - bottomEnd, leftEnd, bottomEnd)` (`:2597`).
        [x, y + h - bottom, left, bottom],
        // `new Rectangle(x + leftEnd, y + h - bottomEnd, w - ..., bottomEnd)` (`:2598`).
        [x + left, y + h - bottom, mid_w, bottom],
        // `new Rectangle(x + w - rightEnd, y + h - bottomEnd, rightEnd, bottomEnd)` (`:2599`).
        [x + w - right, y + h - bottom, right, bottom],
    ];
    let src = [
        [0.0, 0.0, left, top],                          // (`:2591`)
        [left, 0.0, smid_w, top],                       // (`:2592`)
        [sheet_w - right, 0.0, right, top],             // (`:2593` — source is the normal top-right slice)
        [0.0, top, left, smid_h],                       // (`:2594`)
        [left, top, smid_w, smid_h],                    // (`:2595`)
        [sheet_w - right, top, right, smid_h],          // (`:2596`)
        [0.0, sheet_h - bottom, left, bottom],          // (`:2597`)
        [left, sheet_h - bottom, smid_w, bottom],       // (`:2598`)
        [sheet_w - right, sheet_h - bottom, right, bottom], // (`:2599`)
    ];
    (dst, src)
}

/// One atlas sprite from a sub-rectangle `src` (the sheet's own pixels) at `dst`
/// (`[x, y, w, h]`). Zero-sized pieces draw nothing (`Utils.DrawSplicedPanel`
/// hands XNA degenerate rectangles at the clamped 20 px boxes; XNA skips them).
fn push_sheet(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &SheetRect,
    dst: [f32; 4],
    src: [f32; 4],
    color: [f32; 4],
) {
    if dst[2] <= 0.0 || dst[3] <= 0.0 {
        return;
    }
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let x0 = sheet.x as f32 + src[0];
    let y0 = sheet.y as f32 + src[1];
    let uv = [x0 / aw, y0 / ah, (x0 + src[2]) / aw, (y0 + src[3]) / ah];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, color);
}

/// `Utils.DrawSplicedPanel` (`Utils.cs:2581-2600`, the draw). `sheet == None`
/// draws nothing, so `Lookups::default()` is safe (Review Focus 7).
#[allow(dead_code)] // Consumed by Tasks 3-4 (the card panels) and 7-8.
#[allow(clippy::too_many_arguments)]
pub fn draw_spliced_panel(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<SheetRect>,
    rect: [f32; 4],
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    color: [f32; 4],
) {
    let Some(sheet) = sheet else { return };
    let (dst, src) = spliced_rects(
        rect,
        left,
        right,
        top,
        bottom,
        sheet.width as f32,
        sheet.height as f32,
    );
    for i in 0..9 {
        push_sheet(r, lookups, &sheet, dst[i], src[i], color);
    }
}

// --- UITextPanel -------------------------------------------------------------

/// `UIPanel.BorderColor = Color.Black` (`UIPanel.cs:18`).
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub const TEXT_PANEL_BORDER: [u8; 4] = [0, 0, 0, 255];

/// `UIPanel.BackgroundColor = new Color(63, 82, 151) * 0.7f` (`UIPanel.cs:20`),
/// XNA byte truncation: `(44, 57, 105, 178)`.
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub const TEXT_PANEL_BG: [u8; 4] = [44, 57, 105, 178];

/// `FadedMouseOver`'s panel background, `new Color(73, 94, 171)`
/// (`UICharacterSelect.cs:125`).
#[allow(dead_code)] // Consumed by Tasks 3-4.
pub const FADED_MOUSE_OVER_BG: [u8; 4] = [73, 94, 171, 255];

/// `UITextPanel<T>` (`UITextPanel.cs`): a 9-slice `UIPanel` (default
/// `BackgroundColor = (63,82,151)*0.7`, `BorderColor = Black`, `UIPanel.cs:18,20`)
/// plus the large-font label. `text_w` is the caller-measured **unscaled**
/// string width — `Font.MeasureString(text).X` on the **Death** font when
/// `large`, the Mouse font otherwise (`UITextPanel.cs:126-131`); every use of
/// it applies `_textScale` exactly once, like `_textSize` does (`:132`).
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub struct TextPanelStyle {
    /// `_textScale` (`UITextPanel.cs:12`).
    pub text_scale: f32,
    /// `_isLarge` — Death font when true, Mouse font otherwise (`:126-131`).
    pub large: bool,
    /// `SetPadding` (`UIPanel.cs:30-33`; the panel default is the 12 px corner,
    /// the P6 text panels set 15).
    pub padding: f32,
    /// `BackgroundColor` (pre-tint bytes; `UIPanel.cs:20`).
    pub bg: [u8; 4],
    /// `BorderColor` (pre-tint bytes; `UIPanel.cs:18`).
    pub border: [u8; 4],
}

/// The `SetText` minimum sizes (`UITextPanel.cs:132-134`): `MinWidth =
/// _textSize.X + PaddingLeft + PaddingRight`, `MinHeight =
/// (32 if large else 16)*scale + PaddingTop + PaddingBottom`, applied to the
/// caller-resolved hint box (the rect the layout pass produced). The C# clamps
/// the *width* through the layout pass; here the hint box already carries the
/// width the caller wants, so only the `SetText` minimums can grow it.
#[allow(dead_code)] // Consumed by Tasks 3-8.
pub fn text_panel_rect(style: &TextPanelStyle, text_w: f32, box_hint: [f32; 4]) -> [f32; 4] {
    // `this._textSize = new Vector2(MeasureString(text).X, large ? 32f : 16f) * textScale;` (`:132`).
    let text_size_x = text_w * style.text_scale;
    let text_h = (if style.large { 32.0 } else { 16.0 }) * style.text_scale;
    let min_w = text_size_x + style.padding * 2.0; // `:133`
    let min_h = text_h + style.padding * 2.0; // `:134`
    [
        box_hint[0],
        box_hint[1],
        box_hint[2].max(min_w),
        box_hint[3].max(min_h),
    ]
}

/// `DynamicSpriteFont.DrawString` through the panel's font slot (`UITextPanel.cs:126-131`):
/// Death font for `large`, Mouse font otherwise; the built-in bitmap font when
/// the retail fonts are absent (`Lookups::default()`, Review Focus 7).
fn panel_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    large: bool,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    let font = if large {
        lookups.death_font.as_ref()
    } else {
        lookups.ui_font.as_ref()
    };
    match font {
        Some(f) => push_font_text(r, f, x, y, scale, color, text),
        None => r.push_text(x, y, scale, color, text),
    }
}

/// `UITextPanel.DrawSelf` (`UITextPanel.cs:139-146`) + `DrawText` (`:161-169`):
/// the `UIPanel` 9-slice (background then border, `UIPanel.cs:68-76`), then the
/// label. `hover` swaps the colours to `FadedMouseOver`
/// (`UICharacterSelect.cs:122-127`: bg `(73, 94, 171)`, border
/// `Colors.FancyUIFatButtonMouseOver`); idle keeps `style`'s own pair (the
/// `FadedMouseOut` restore, `:129-133`). Missing sheets/fonts draw what they
/// can and panic nowhere (Review Focus 7).
#[allow(dead_code)] // Consumed by Tasks 3-8.
#[allow(clippy::too_many_arguments)]
pub fn draw_text_panel(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    style: &TextPanelStyle,
    text: &str,
    text_w: f32,
    hover: bool,
) {
    // `UIPanel.DrawSelf` (`UIPanel.cs:68-76`), coloured by the hover state.
    let bg = if hover { FADED_MOUSE_OVER_BG } else { style.bg };
    let border = if hover {
        FANCY_UI_FAT_BUTTON_MOUSE_OVER
    } else {
        style.border
    };
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, rect, super::color::tint(bg));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, rect, super::color::tint(border));

    // `TextDrawPosition` (`UITextPanel.cs:92-108`): centred horizontally inside
    // the inner dimensions (`TextHAlign = 0.5`, `:22`) against the scaled
    // `_textSize.X` (`:132`), then shifted up by `2*scale` (small, `:105`) or
    // `10*scale*scale` (large, `:101`).
    let pad = style.padding;
    let inner = inner(rect, [pad, pad, pad, pad]);
    let text_size_x = text_w * style.text_scale;
    let tx = inner[0] + (inner[2] - text_size_x) * 0.5;
    let ty = inner[1]
        - if style.large {
            10.0 * style.text_scale * style.text_scale
        } else {
            2.0 * style.text_scale
        };
    // `DrawText` (`:161-169`): `TextColor` defaults to White (`:18`).
    let white = [1.0, 1.0, 1.0, 1.0];
    if style.large {
        // `Utils.DrawBorderStringBig` (`Utils.cs:2527-2534`): the 8-way black
        // border, then the colour on top.
        for dy in -1i32..=1 {
            for dx in -1i32..=1 {
                if dx != 0 || dy != 0 {
                    panel_text(
                        r,
                        lookups,
                        true,
                        tx + dx as f32,
                        ty + dy as f32,
                        style.text_scale,
                        [0.0, 0.0, 0.0, 1.0],
                        text,
                    );
                }
            }
        }
        panel_text(r, lookups, true, tx, ty, style.text_scale, white, text);
    } else {
        // `Utils.DrawBorderString` (`Utils.cs:2510-2518`): the ChatManager
        // shadow, then the colour — the port's 1 px black stand-in.
        panel_text(
            r,
            lookups,
            false,
            tx + 1.0,
            ty + 1.0,
            style.text_scale,
            [0.0, 0.0, 0.0, 1.0],
            text,
        );
        panel_text(r, lookups, false, tx, ty, style.text_scale, white, text);
    }
}

// --- UIImageButton -----------------------------------------------------------

/// `UIImageButton.DrawSelf` (`UIImageButton.cs:48-56`): the sheet at `rect`
/// (the rect is the caller's — the button is texture-sized, `:30-33`) tinted
/// `Color * (hover ? 1.0 : 0.4)` (`:51`); the hover border sheet drawn *over*
/// the base at full tint (`BorderColor`, `:52-55`). `None` sheets draw nothing.
#[allow(dead_code)] // Consumed by Tasks 3-4 (the card buttons).
pub fn draw_image_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<SheetRect>,
    rect: [f32; 4],
    hover_sheet: Option<SheetRect>,
    hover: bool,
    tint: [f32; 4],
) {
    // `this.Color * (base.IsMouseHovering ? this._visibilityActive : this._visibilityInactive)`
    // (`:51`); `_visibilityActive = 1f`, `_visibilityInactive = 0.4f` (`:13-15`).
    let base = if hover {
        tint
    } else {
        [
            tint[0] * 0.4,
            tint[1] * 0.4,
            tint[2] * 0.4,
            tint[3] * 0.4,
        ]
    };
    if let Some(sheet) = sheet {
        let uv = lookups.hud_uv(Some(&sheet)).unwrap_or([0.0; 4]);
        r.push_ui_sprite([rect[0], rect[1]], [rect[2], rect[3]], uv, base);
    }
    // `if (this._borderTexture != null && base.IsMouseHovering)` (`:52-55`).
    if hover {
        if let Some(border) = hover_sheet {
            let uv = lookups.hud_uv(Some(&border)).unwrap_or([0.0; 4]);
            r.push_ui_sprite([rect[0], rect[1]], [rect[2], rect[3]], uv, tint);
        }
    }
}

// --- UIColoredImageButton ----------------------------------------------------

/// One centre-origin sprite from a whole sheet (`UIColoredImageButton.cs:72-89`):
/// `spriteBatch.Draw(tex, position, null, color, 0f, tex.Size() / 2f, 1f, ...)`
/// with `position = dimensions.Position() + dimensions.Size() / 2f` (`:71`).
fn push_centred(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &SheetRect,
    cx: f32,
    cy: f32,
    color: [f32; 4],
) {
    let uv = lookups.hud_uv(Some(sheet)).unwrap_or([0.0; 4]);
    let w = sheet.width as f32;
    let h = sheet.height as f32;
    r.push_ui_sprite([cx - w * 0.5, cy - h * 0.5], [w, h], uv, color);
}

/// `UIColoredImageButton.DrawSelf` (`UIColoredImageButton.cs:68-90`): the back
/// panel, then the border sheet when **hovered** (`:74-77`), then the highlight
/// sheet when **selected** (`:78-81`), then the middle texture, then the tinted
/// icon — all centre-origin at `rect`'s centre, scale 1. (The plan's comment
/// swaps the border/highlight predicates; the C# has border = hovered,
/// highlight = selected — followed here.) The back panel dims to
/// `Color.White * 0.4` when not hovered (`:72`); the border/highlight/middle/
/// icon draw at full strength. Missing sheets draw nothing (Review Focus 7).
#[allow(dead_code)] // Consumed by Tasks 7-8 (categories/options).
#[allow(clippy::too_many_arguments)]
pub fn draw_colored_image_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    small: bool,
    rect: [f32; 4],
    middle: Option<SheetRect>,
    icon: Option<SheetRect>,
    icon_color: [u8; 4],
    selected: bool,
    hovered: bool,
) {
    // `Vector2 position = dimensions.Position() + new Vector2(dimensions.Width,
    // dimensions.Height) / 2f;` (`:71`).
    let cx = rect[0] + rect[2] * 0.5;
    let cy = rect[1] + rect[3] * 0.5;
    // The back panel: SmallPanel 32x32 vs CategoryPanel 44x44 (`:35-42`).
    let back = if small { lookups.cc_small_panel } else { lookups.cc_panel };
    if let Some(back) = back {
        // `Color.White * (IsMouseHovering ? 1f : 0.4f)` (`:72`).
        let a = if hovered { 1.0 } else { 0.4 };
        push_centred(r, lookups, &back, cx, cy, [a, a, a, a]);
    }
    if hovered {
        // `:74-77` — SmallPanelBorder vs CategoryPanelBorder (`:45-52`).
        let border = if small {
            lookups.cc_small_panel_border
        } else {
            lookups.cc_panel_border
        };
        if let Some(border) = border {
            push_centred(r, lookups, &border, cx, cy, [1.0; 4]);
        }
    }
    if selected {
        // `:78-81` — CategoryPanelHighlight (both sizes, `:45`).
        if let Some(highlight) = lookups.cc_panel_highlight {
            push_centred(r, lookups, &highlight, cx, cy, [1.0; 4]);
        }
    }
    if let Some(middle) = middle {
        push_centred(r, lookups, &middle, cx, cy, [1.0; 4]);
    }
    if let Some(icon) = icon {
        push_centred(r, lookups, &icon, cx, cy, super::color::tint(icon_color));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values hand-derived from the C# (`UIElement.cs:306-323`,
    // `Utils.cs:2581-2600`, `UITextPanel.cs:92-134`, `Main.cs:66748-66768`,
    // `ID/Colors.cs:157-159`) — never from this code.

    const ROOT: [f32; 4] = [475.0, 220.0, 650.0, 680.0]; // `UICharacterSelect.cs:38-42`

    #[test]
    fn resolve_lands_the_select_screen_children() {
        // The New button: `Width.Set(-10f, 0.5f)` + `HAlign = 1f`
        // (`UICharacterSelect.cs:76-84`): w = 0.5*650 - 10 = 315,
        // x = 475 + (650 - 315) = 810. Height 50, `Top` untouched (0).
        let new_button = resolve(
            ROOT,
            Style::px(0.0),
            Style::px(0.0),
            Style::new(-10.0, 0.5),
            Style::px(50.0),
            1.0,
            0.0,
            0.0,
            0.0,
            ROOT[2],
            [0.0; 4],
        );
        assert_eq!(new_button, [810.0, 220.0, 315.0, 50.0]);

        // The same width centred (h_align 0.5): x = 475 + 0.5*(650-315) = 642.5.
        let centred = resolve(
            ROOT,
            Style::px(0.0),
            Style::px(0.0),
            Style::new(-10.0, 0.5),
            Style::px(50.0),
            0.5,
            0.0,
            0.0,
            0.0,
            ROOT[2],
            [0.0; 4],
        );
        assert_eq!(centred, [642.5, 220.0, 315.0, 50.0]);

        // The Back button: `Top.Set(-45f)` + `VAlign = 1f` (`:66-74`) —
        // y = 220 + (680 - 50) - 45 = 805.
        let back = resolve(
            ROOT,
            Style::px(0.0),
            Style::px(-45.0),
            Style::new(-10.0, 0.5),
            Style::px(50.0),
            0.0,
            1.0,
            0.0,
            0.0,
            ROOT[2],
            [0.0; 4],
        );
        assert_eq!(back, [475.0, 805.0, 315.0, 50.0]);
    }

    #[test]
    fn resolve_clamps_min_width_before_aligning() {
        // A 100 px-wide element with MinWidth 315 grows first, then aligns:
        // x = 475 + (650 - 315) = 810 — not 475 + (650 - 100) = 1025.
        let clamped = resolve(
            ROOT,
            Style::px(0.0),
            Style::px(0.0),
            Style::px(100.0),
            Style::px(50.0),
            1.0,
            0.0,
            315.0,
            0.0,
            ROOT[2],
            [0.0; 4],
        );
        assert_eq!(clamped, [810.0, 220.0, 315.0, 50.0]);

        // MaxWidth shrinks too (the root's own `MaxWidth.Set(650f, 0f)` case,
        // `UICharacterSelect.cs:38-42`): a 1280-wide 80% root stays 650.
        let root = resolve(
            [0.0, 0.0, 1600.0, 900.0], // the UserInterface's screen rect
            Style::px(0.0),            // no Left set (`StyleDimension.Empty`)
            Style::px(220.0),          // `Top.Set(220f, 0f)`
            Style::new(0.0, 0.8),      // `Width.Set(0f, 0.8f)`
            Style::new(-220.0, 1.0),   // `Height.Set(-220f, 1f)`
            0.5,                       // `HAlign = 0.5f`
            0.0,
            0.0,
            0.0,
            650.0, // `MaxWidth.Set(650f, 0f)`
            [0.0; 4],
        );
        // w = min(1280, 650) = 650, x = 0 + 0.5*(1600 - 650) = 475;
        // h = -220 + 900 = 680, y = 220.
        assert_eq!(root, [475.0, 220.0, 650.0, 680.0]);
    }

    #[test]
    fn resolve_padding_shrinks_the_inner_rect_only() {
        let outer = resolve(
            ROOT,
            Style::px(0.0),
            Style::px(0.0),
            Style::px(315.0),
            Style::px(50.0),
            1.0,
            0.0,
            0.0,
            0.0,
            ROOT[2],
            [12.0; 4],
        );
        assert_eq!(outer, [810.0, 220.0, 315.0, 50.0]);
        // `UIPanel.cs:32`'s SetPadding(12): the inner rect the children see.
        assert_eq!(inner(outer, [12.0; 4]), [822.0, 232.0, 291.0, 26.0]);
        // Asymmetric padding (the `right` column's PaddingTop 8 / Bottom 12,
        // `UIWorldCreation.cs:187-197`).
        assert_eq!(inner([550.0, 202.0, 500.0, 298.0], [0.0, 8.0, 0.0, 12.0]), [550.0, 210.0, 500.0, 278.0]);
    }

    #[test]
    fn spliced_rects_match_draw_spliced_panel() {
        // `Utils.DrawSplicedPanel(sb, tex, 100, 200, 150, 120, 10, 10, 10, 10, c)`
        // with the sheet at the same 150x120 size as the box (the plan's table).
        let (dst, src) =
            spliced_rects([100.0, 200.0, 150.0, 120.0], 10.0, 10.0, 10.0, 10.0, 150.0, 120.0);
        assert_eq!(dst[0], [100.0, 200.0, 10.0, 10.0]); // TL
        assert_eq!(src[0], [0.0, 0.0, 10.0, 10.0]);
        assert_eq!(dst[1], [110.0, 200.0, 130.0, 10.0]); // top edge
        assert_eq!(src[1], [10.0, 0.0, 130.0, 10.0]); // `texture.Width - 20`
        // The `Utils.cs:2593` quirk: the top-right corner is drawn with the
        // destination size `(topEnd, rightEnd)` — at 10/10/10/10 it is
        // shape-neutral, but the formula is literal.
        assert_eq!(dst[2], [240.0, 200.0, 10.0, 10.0]);
        assert_eq!(src[2], [140.0, 0.0, 10.0, 10.0]); // `texture.Width - rightEnd`
        assert_eq!(dst[3], [100.0, 210.0, 10.0, 100.0]); // left edge
        assert_eq!(src[3], [0.0, 10.0, 10.0, 100.0]);
        assert_eq!(dst[4], [110.0, 210.0, 130.0, 100.0]); // centre
        assert_eq!(src[4], [10.0, 10.0, 130.0, 100.0]);
        assert_eq!(dst[5], [240.0, 210.0, 10.0, 100.0]); // right edge
        assert_eq!(src[5], [140.0, 10.0, 10.0, 100.0]);
        assert_eq!(dst[6], [100.0, 310.0, 10.0, 10.0]); // BL
        assert_eq!(src[6], [0.0, 110.0, 10.0, 10.0]);
        assert_eq!(dst[7], [110.0, 310.0, 130.0, 10.0]); // bottom edge
        assert_eq!(src[7], [10.0, 110.0, 130.0, 10.0]);
        assert_eq!(dst[8], [240.0, 310.0, 10.0, 10.0]); // BR
        assert_eq!(src[8], [140.0, 110.0, 10.0, 10.0]);
    }

    #[test]
    fn spliced_rects_top_right_quirk_is_visible_when_asymmetric() {
        // With distinct ends the quirk shows: the top-right corner is `top`
        // wide and `right` tall, while the right edge below it keeps `right`.
        let (dst, src) =
            spliced_rects([0.0, 0.0, 100.0, 100.0], 8.0, 12.0, 6.0, 14.0, 100.0, 100.0);
        assert_eq!(dst[2], [88.0, 0.0, 6.0, 12.0]); // `:2593` — (topEnd, rightEnd)
        assert_eq!(src[2], [88.0, 0.0, 12.0, 6.0]); // the source stays (rightEnd, topEnd)
        assert_eq!(dst[5], [88.0, 6.0, 12.0, 80.0]); // the right edge is `right` wide
    }

    #[test]
    fn spliced_rects_on_the_44x44_sheet_map_one_to_one() {
        // `PanelGrayscale` spliced at 10/10/10/10 (`GroupOptionButton.cs:126`):
        // with the box equal to the sheet, dest and source agree piecewise.
        let (dst, src) = spliced_rects([0.0, 0.0, 44.0, 44.0], 10.0, 10.0, 10.0, 10.0, 44.0, 44.0);
        assert_eq!(dst[1], [10.0, 0.0, 24.0, 10.0]);
        assert_eq!(src[1], [10.0, 0.0, 24.0, 10.0]);
        assert_eq!(dst[4], [10.0, 10.0, 24.0, 24.0]);
        assert_eq!(src[4], [10.0, 10.0, 24.0, 24.0]);
        assert_eq!(dst[8], [34.0, 34.0, 10.0, 10.0]);
        assert_eq!(src[8], [34.0, 34.0, 10.0, 10.0]);
    }

    #[test]
    fn spliced_rects_clamp_the_box_to_the_ends() {
        // `Utils.cs:2582-2587`: a 12 px box with 10 px ends becomes 20 px, so
        // the middle pieces collapse to zero width/height (and draw nothing).
        let (dst, _) = spliced_rects([0.0, 0.0, 12.0, 12.0], 10.0, 10.0, 10.0, 10.0, 24.0, 24.0);
        assert_eq!(dst[1][2], 0.0); // top edge squeezed away
        assert_eq!(dst[4], [10.0, 10.0, 0.0, 0.0]); // centre squeezed away
        assert_eq!(dst[8], [10.0, 10.0, 10.0, 10.0]); // BR sits at the clamped box's corner
    }

    #[test]
    fn colour_table_matches_the_vanilla_literals() {
        // `ID/Colors.cs:157-159` + `Main.cs:66748,66765-66768`.
        assert_eq!(FANCY_UI_FAT_BUTTON_MOUSE_OVER, OUR_FAVORITE_COLOR);
        assert_eq!(OUR_FAVORITE_COLOR, [255, 231, 69, 255]);
        assert_eq!(INVENTORY_DEFAULT_COLOR, [63, 65, 151, 255]);
        assert_eq!(MC_COLOR, [255, 153, 0, 255]);
        assert_eq!(HC_COLOR, [255, 38, 25, 255]);
        assert_eq!(CREATIVE_MODE_COLOR, [255, 120, 187, 255]);
        assert_eq!(LEGENDARY_MODE_COLOR, [50, 205, 50, 255]);
        assert_eq!(CREATION_SEPARATOR_COLOR, [81, 83, 149, 229]);
    }

    #[test]
    fn colour_bytes_are_recomputed_through_the_xna_maths() {
        // Truncating float->byte conversions (positive values only here, so
        // Rust's `as u8` matches C#'s `(byte)`).
        // `Main.hcColor = new Color(1f, 0.15f, 0.1f)` (`Main.cs:66766`).
        let byte = |v: f32| v as u8;
        assert_eq!(
            [byte(1.0 * 255.0), byte(0.15f32 * 255.0), byte(0.1f32 * 255.0)],
            [255, 38, 25]
        );
        // `Main.creativeModeColor = Color.Lerp(HotPink, White, 0.1f)`
        // (`Main.cs:66767`); HotPink = (255, 105, 180).
        let lerp = |a: f32, b: f32| (a + (b - a) * 0.1f32) as u8;
        assert_eq!(
            [lerp(255.0, 255.0), lerp(105.0, 255.0), lerp(180.0, 255.0)],
            [255, 120, 187]
        );
        // The separator: `Lerp(White, (63,65,151), 0.85)` truncates to bytes
        // FIRST, then `* 0.9` truncates again (`UICharacterCreation.cs:448`).
        let lerp85 = |a: f32, b: f32| (a + (b - a) * 0.85f32) as u8;
        let step1 = [
            lerp85(255.0, 63.0),
            lerp85(255.0, 65.0),
            lerp85(255.0, 151.0),
            255u8,
        ];
        assert_eq!(step1, [91, 93, 166, 255]);
        assert_eq!(super::super::color::mul(step1, 0.9), CREATION_SEPARATOR_COLOR);
        // `UIPanel.BackgroundColor = (63, 82, 151) * 0.7f` (`UIPanel.cs:20`).
        assert_eq!(TEXT_PANEL_BG, super::super::color::mul([63, 82, 151, 255], 0.7));
    }

    #[test]
    fn text_panel_rect_applies_the_settext_minimums_to_the_hint_box() {
        // The Select Player header: `0.8f` large with `SetPadding(15f)`
        // (`UICharacterSelect.cs:59-64`), hint box `(475, 180, 650, 55.6)`.
        let header = TextPanelStyle {
            text_scale: 0.8,
            large: true,
            padding: 15.0,
            bg: FADED_MOUSE_OVER_BG,
            border: TEXT_PANEL_BORDER,
        };
        // MinWidth = 100*0.8 + 15 + 15 = 110 <= 650; MinHeight = 32*0.8 + 30 =
        // 55.6 — the hint box already satisfies both (`UITextPanel.cs:133-134`).
        assert_eq!(
            text_panel_rect(&header, 100.0, [475.0, 180.0, 650.0, 55.6]),
            [475.0, 180.0, 650.0, 55.6]
        );
        // A too-narrow hint grows to MinWidth.
        assert_eq!(
            text_panel_rect(&header, 100.0, [475.0, 180.0, 100.0, 55.6]),
            [475.0, 180.0, 110.0, 55.6]
        );
        // A too-short hint grows to MinHeight (the 55.6 the layout table pins).
        assert_eq!(
            text_panel_rect(&header, 100.0, [475.0, 180.0, 650.0, 40.0]),
            [475.0, 180.0, 650.0, 55.6]
        );
        // The 50 px Back/New buttons (`0.7f` large, padding 12): MinHeight =
        // 32*0.7 + 24 = 46.4 < 50, MinWidth for "Back" at ~70 px is 94 <= 315.
        let button = TextPanelStyle {
            text_scale: 0.7,
            large: true,
            padding: 12.0,
            bg: TEXT_PANEL_BG,
            border: TEXT_PANEL_BORDER,
        };
        assert_eq!(
            text_panel_rect(&button, 70.0, [475.0, 805.0, 315.0, 50.0]),
            [475.0, 805.0, 315.0, 50.0]
        );
    }

    #[test]
    fn xna_clamp_keeps_the_min_when_min_exceeds_max() {
        // `MathHelper.Clamp` = `Math.Max(min, Math.Min(max, value))` — with
        // min > max XNA returns min; Rust's `f32::clamp` would panic.
        assert_eq!(xna_clamp(50.0, 100.0, 20.0), 100.0);
        assert_eq!(xna_clamp(10.0, 0.0, 20.0), 10.0);
        assert_eq!(xna_clamp(30.0, 0.0, 20.0), 20.0);
        assert_eq!(xna_clamp(-5.0, 0.0, 20.0), 0.0);
    }

    #[test]
    fn lookups_default_keeps_every_p6_sheet_none() {
        // Review Focus 7: `Lookups::default()` is all-`None`; the P6 fields
        // exist and are empty, so every draw helper's `Option` guard holds.
        let l = assets::Lookups::default();
        assert!(l.card_buttons.iter().all(|s| s.is_none()));
        assert!(l.card_cloud.iter().all(|s| s.is_none()));
        assert!(l.ui_divider.is_none());
        assert!(l.ui_inner_panel.is_none());
        assert!(l.ui_player_background.is_none());
        assert!(l.ui_world_icon.is_none());
        assert!(l.ui_icon_completion.is_none());
        assert!(l.ui_icon_mixed_seed.is_none());
        assert!(l.ui_icon_played_before.is_none());
        assert!(l.ui_icon_newly_generated.is_none());
        assert!(l.cc_panel.is_none());
        assert!(l.cc_panel_border.is_none());
        assert!(l.cc_panel_highlight.is_none());
        assert!(l.cc_panel_gray.is_none());
        assert!(l.cc_small_panel.is_none());
        assert!(l.cc_small_panel_border.is_none());
        assert!(l.cc_char_info.is_none());
        assert!(l.cc_hair_style.is_none());
        assert!(l.cc_hair_arrow.is_none());
        assert!(l.cc_colors.iter().all(|s| s.is_none()));
        assert!(l.cc_copy_paste_random.iter().all(|s| s.is_none()));
        assert!(l.cc_cloth_styles.iter().all(|s| s.is_none()));
        assert!(l.cc_voice.is_none());
        assert!(l.wc_preview_border.is_none());
        assert!(l.wc_icons.iter().all(|s| s.is_none()));
        assert!(l.wc_size_icons.iter().all(|s| s.is_none()));
        assert!(l.wc_difficulty_icons.iter().all(|s| s.is_none()));
        assert!(l.wc_evil_icons.iter().all(|s| s.is_none()));
        assert!(l.wc_preview_bg.iter().all(|s| s.is_none()));
        assert!(l.wc_preview_bunny.iter().all(|s| s.is_none()));
        assert!(l.wc_preview_evil.iter().all(|s| s.is_none()));
        assert!(l.wc_preview_size.iter().all(|s| s.is_none()));
        assert!(l.vk_shift.is_none());
        assert!(l.vk_backspace.is_none());
    }
}
