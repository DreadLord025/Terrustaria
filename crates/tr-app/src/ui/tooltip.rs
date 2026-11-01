//! Item tooltip (`Main.MouseText`, `Utils.DrawInvBG`). Filled in by Task 4 of the
//! Whole-UI P1 plan.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::UiPulse;
use super::text::{hud_text, hud_text_width};
use crate::assets;
use crate::item::{HOTBAR, INV, ItemSlot};
use crate::registry;

/// `toolTipDistance` (`Main.cs:407`, set to 6 at `Main.cs:66563`).
const TOOL_TIP_DISTANCE: f32 = 6.0;
/// Opaque-box margin (`num4 = 18`, `Main.cs:20385`) used for the on-screen clamp.
const BOX_MARGIN: f32 = 18.0;
/// Opaque box offset (`X += 8; Y += 2`, `Main.cs:20383-20384`).
const BOX_OFFSET: Vec2 = Vec2::new(8.0, 2.0);
/// Box left/top inset from the text origin (`num8 = 14`, `num9 = 9`, `Main.cs:20400-20401`).
const BOX_INSET: Vec2 = Vec2::new(14.0, 9.0);
/// Box height padding: int `num9 + num9/2` = `9 + 4` = 13 (`Main.cs:20402`).
const BOX_HEIGHT_PAD: f32 = 13.0;
/// `Main.cs:20402`: `new Color(23, 25, 81, 255) * 0.925f`.
const TOOLTIP_BG: [f32; 4] = [
    23.0 / 255.0 * 0.925,
    25.0 / 255.0 * 0.925,
    81.0 / 255.0 * 0.925,
    0.925,
];

/// The nine destination rectangles of `Utils.DrawInvBG` (`Utils.cs:2548-2572`): 10 px
/// corners, the middle/corner cells stretched to fill, clamped to a 20x20 minimum
/// (`Utils.cs:2555-2562`).
pub fn inv_bg_rects(rect: [f32; 4]) -> [[f32; 4]; 9] {
    let [x, y, mut w, mut h] = rect;
    if w < 20.0 {
        w = 20.0;
    }
    if h < 20.0 {
        h = 20.0;
    }
    [
        [x, y, 10.0, 10.0],
        [x + 10.0, y, w - 20.0, 10.0],
        [x + w - 10.0, y, 10.0, 10.0],
        [x, y + 10.0, 10.0, h - 20.0],
        [x + 10.0, y + 10.0, w - 20.0, h - 20.0],
        [x + w - 10.0, y + 10.0, 10.0, h - 20.0],
        [x, y + h - 10.0, 10.0, 10.0],
        [x + 10.0, y + h - 10.0, w - 20.0, 10.0],
        [x + w - 10.0, y + h - 10.0, 10.0, 10.0],
    ]
}

/// One 10x10 source cell of `Inventory_Back13`, in the texture's own pixels.
fn inv_bg_srcs(w: f32, h: f32) -> [[f32; 4]; 9] {
    // `Utils.cs:2563-2571`: the corners read 10x10 from the four edges; the middle
    // cells read `(10, 10, 10, 10)` and are stretched by the destination rect.
    [
        [0.0, 0.0, 10.0, 10.0],
        [10.0, 0.0, 10.0, 10.0],
        [w - 10.0, 0.0, 10.0, 10.0],
        [0.0, 10.0, 10.0, 10.0],
        [10.0, 10.0, 10.0, 10.0],
        [w - 10.0, 10.0, 10.0, 10.0],
        [0.0, h - 10.0, 10.0, 10.0],
        [10.0, h - 10.0, 10.0, 10.0],
        [w - 10.0, h - 10.0, 10.0, 10.0],
    ]
}

/// Draws the `Inventory_Back13` 9-slice over `rect` (`Utils.DrawInvBG`). Missing the
/// sheet draws nothing.
pub fn draw_inv_bg(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    color: [f32; 4],
) {
    let Some(sheet) = lookups.ui_inv_bg13.as_ref() else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let srcs = inv_bg_srcs(sheet.width as f32, sheet.height as f32);
    let dst = inv_bg_rects(rect);
    for i in 0..9 {
        let [dx, dy, dw, dh] = dst[i];
        if dw <= 0.0 || dh <= 0.0 {
            continue;
        }
        let [sx, sy, sw, sh] = srcs[i];
        let x0 = sheet.x as f32 + sx;
        let y0 = sheet.y as f32 + sy;
        let uv = [
            x0 / aw,
            y0 / ah,
            (x0 + sw) / aw,
            (y0 + sh) / ah,
        ];
        r.push_ui_sprite([dx, dy], [dw, dh], uv, color);
    }
}

/// `Main.MouseText_DrawItemTooltip_GetItemNameColor` (`Main.cs:20557-20620`): the
/// item-rare colour table; `rare == 0` and any unknown value are white.
pub fn item_name_color(rare: i32) -> [f32; 4] {
    fn rgb(r: f32, g: f32, b: f32) -> [f32; 4] {
        [r / 255.0, g / 255.0, b / 255.0, 1.0]
    }
    match rare {
        -11 => rgb(255.0, 175.0, 0.0),
        -1 => rgb(130.0, 130.0, 130.0),
        1 => rgb(150.0, 150.0, 255.0),
        2 => rgb(150.0, 255.0, 150.0),
        3 => rgb(255.0, 200.0, 150.0),
        4 => rgb(255.0, 150.0, 150.0),
        5 => rgb(255.0, 150.0, 255.0),
        6 => rgb(210.0, 160.0, 255.0),
        7 => rgb(150.0, 255.0, 10.0),
        8 => rgb(255.0, 255.0, 10.0),
        9 => rgb(5.0, 200.0, 255.0),
        10 => rgb(255.0, 40.0, 100.0),
        // `rare >= 11` (`Main.cs:20616-20619`). `-12`/`-13` use the dynamic
        // disco/master colours and are not modelled (no such items yet).
        11..=i32::MAX => rgb(180.0, 40.0, 255.0),
        _ => [1.0, 1.0, 1.0, 1.0],
    }
}

/// The tooltip box (`Main.MouseText_DrawItemTooltip`, `Main.cs:20196-20402`): from the
/// mouse, `+14` (`Main.cs:20196`), `+toolTipDistance` (`Main.cs:20378`), then the
/// opaque-box `(8, 2)` offset (`Main.cs:20383-20384`); clamped so the text plus the
/// 18 px margin stays on-screen (`Main.cs:20389-20395`). The returned rect is the
/// `DrawInvBG` box `(X-14, Y-9, w+28, h+13)` (`Main.cs:20402`).
pub fn tooltip_box_rect(mouse: Vec2, w: f32, h: f32, screen: Vec2) -> [f32; 4] {
    let mut x = mouse.x + 14.0 + TOOL_TIP_DISTANCE + BOX_OFFSET.x;
    let mut y = mouse.y + 14.0 + TOOL_TIP_DISTANCE + BOX_OFFSET.y;
    if x + w + BOX_MARGIN > screen.x {
        x = screen.x - w - BOX_MARGIN;
    }
    if y + h + BOX_MARGIN > screen.y {
        y = screen.y - h - BOX_MARGIN;
    }
    [
        x - BOX_INSET.x,
        y - BOX_INSET.y,
        w + BOX_INSET.x * 2.0,
        h + BOX_HEIGHT_PAD,
    ]
}

/// One `MouseText` line height (`DynamicSpriteFont.MeasureString(...).Y`). The built-in
/// bitmap fallback draws 8 px glyphs (`SpriteRenderer::push_text`).
fn line_height(lookups: &assets::Lookups) -> f32 {
    match &lookups.ui_font {
        Some(f) => f.line_spacing,
        None => 8.0,
    }
}

/// `Main.MouseText_DrawItemTooltip` (`Main.cs:20322-20446`): the generic tooltip box
/// plus the item name in its rare colour, with placeholder zeroed stat lines for
/// weapons (spec §9.3; the real `Item.ToolTip` lines are a later data port).
pub fn draw_item_tooltip(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    mouse: Vec2,
    hover: Option<&ItemSlot>,
    screen: Vec2,
    pulse: UiPulse,
) {
    let Some(item) = hover else {
        return;
    };
    // Empty slots show nothing (`Main.HoverItem.type > 0` guard, `Main.cs:20213`).
    if item.count == 0 {
        return;
    }
    // `Main.cs:20358-20360`: every line starts white, line 0 takes the name colour.
    let mut lines: Vec<(&'static str, [f32; 4])> =
        vec![(item.name, item_name_color(registry::rare(item.kind)))];
    if registry::is_weapon(item.kind) {
        for line in [
            "0 melee damage",
            "0 knockback",
            "0% critical strike chance",
            "0 use time",
        ] {
            lines.push((line, [1.0, 1.0, 1.0, 1.0]));
        }
    }
    // `Main.cs:20367-20377`: the widest line and the summed line heights, `num3 = 0`.
    let lh = line_height(lookups);
    let mut w = 0.0f32;
    let mut h = 0.0f32;
    for (text, _) in &lines {
        w = w.max(hud_text_width(r, lookups, text, 1.0));
        h += lh;
    }
    let rect = tooltip_box_rect(mouse, w, h, screen);
    draw_inv_bg(r, lookups, rect, TOOLTIP_BG);
    // Text origin: the box's top-left plus the `(14, 9)` inset (`Main.cs:20400-20402`).
    let tx = rect[0] + BOX_INSET.x;
    let ty = rect[1] + BOX_INSET.y;
    // `Main.cs:20406-20407`: each line colour is multiplied by `mouseTextColor/255`
    // with the raw `mouseTextColor` as its alpha.
    let tint = pulse.text;
    let mut line_y = 0.0;
    for (text, color) in &lines {
        let line = [
            color[0] * tint,
            color[1] * tint,
            color[2] * tint,
            tint,
        ];
        // `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333-344`):
        // four black copies at `shadowDirection * spread` (`spread = 2`) under the text.
        for dir in [(-2.0f32, 0.0f32), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
            hud_text(
                r,
                lookups,
                tx + dir.0,
                ty + line_y + dir.1,
                1.0,
                [0.0, 0.0, 0.0, tint],
                text,
            );
        }
        hud_text(r, lookups, tx, ty + line_y, 1.0, line, text);
        line_y += lh;
    }
}

/// Index of the slot drawn under `mouse`. Open inventory: the 10x5 grid at
/// `inventoryScale = 0.85` (`Main.DrawInventory`). Closed: the hotbar, whose slots are
/// 52x52 scaled by `hotbarScale` and strided by 4 px (`GUIHotbarDrawInner`,
/// `Main.cs:43613-43639`). Empty slots are not hoverable (`Main.cs:20213`).
pub fn hovered_slot(
    mouse: Vec2,
    inventory_open: bool,
    _selected: usize,
    hotbar_scales: &[f32; HOTBAR],
    inventory: &[ItemSlot; INV],
) -> Option<usize> {
    let hit = |x: f32, y: f32, s: f32| {
        mouse.x >= x && mouse.x <= x + s && mouse.y >= y && mouse.y <= y + s
    };
    if inventory_open {
        let inv_scale = 0.85f32;
        let slot = 52.0 * inv_scale;
        for j in 0..5usize {
            for i in 0..10usize {
                let idx = i + j * 10;
                let x = 20.0 + i as f32 * 56.0 * inv_scale;
                let y = 20.0 + j as f32 * 56.0 * inv_scale;
                if hit(x, y, slot) {
                    return (inventory[idx].count > 0).then_some(idx);
                }
            }
        }
    } else {
        let mut x = 20.0;
        for i in 0..HOTBAR {
            let scale = hotbar_scales[i].clamp(0.75, 1.0);
            let slot = 52.0 * scale;
            let y = 20.0 + 22.0 * (1.0 - scale);
            if hit(x, y, slot) {
                return (inventory[i].count > 0).then_some(i);
            }
            x += slot + 4.0;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inv_bg_min_size_is_twenty() {
        let r = inv_bg_rects([100.0, 100.0, 5.0, 8.0]);
        assert_eq!(r[0], [100.0, 100.0, 10.0, 10.0]);
        // and the nine pieces tile the clamped 20x20 rect: r[8] == [110.0, 110.0, 10.0, 10.0]
        assert_eq!(r[8], [110.0, 110.0, 10.0, 10.0]);
    }

    #[test]
    fn item_name_color_zero_is_white_neg_one_is_grey() {
        assert_eq!(item_name_color(0), [1.0, 1.0, 1.0, 1.0]);
        assert_eq!(item_name_color(-1), [130.0 / 255.0, 130.0 / 255.0, 130.0 / 255.0, 1.0]);
    }

    #[test]
    fn item_name_color_covers_high_and_negative_rarities() {
        // `rare >= 11` is the purple expert/master colour (`Main.cs:20616`).
        assert_eq!(
            item_name_color(11),
            [180.0 / 255.0, 40.0 / 255.0, 1.0, 1.0]
        );
        assert_eq!(item_name_color(99), item_name_color(11));
        // `rare == -11` is the gold quest colour (`Main.cs:20568`).
        assert_eq!(
            item_name_color(-11),
            [1.0, 175.0 / 255.0, 0.0, 1.0]
        );
    }

    #[test]
    fn tooltip_clamps_to_the_screen() {
        let r = tooltip_box_rect(Vec2::new(1590.0, 880.0), 200.0, 40.0, Vec2::new(1600.0, 900.0));
        assert!(r[0] + r[2] <= 1600.0 && r[1] + r[3] <= 900.0);
    }

    #[test]
    fn hovered_slot_none_when_not_over_a_cell() {
        let inv: [ItemSlot; INV] = std::array::from_fn(|_| ItemSlot { kind: 0, name: "", count: 0 });
        assert_eq!(hovered_slot(Vec2::new(2.0, 2.0), false, 0, &[1.0; HOTBAR], &inv), None);
    }
}
