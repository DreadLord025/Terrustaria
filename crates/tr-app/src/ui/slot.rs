//! Inventory and hotbar slots: `ItemSlot.Draw` and the hotbar digits.

use tr_render::SpriteRenderer;

use super::text::{hud_item_text, hud_item_text_width};
use crate::assets;
use crate::ItemSlot;

/// `ItemSlot.Draw`: the item's own texture at inventory scale, centred in the slot,
/// with the stack size in the bottom-right corner.
pub(super) fn draw_slot_item(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    item: &ItemSlot,
    x: f32,
    y: f32,
    slot: f32,
    scale: f32,
) {
    if item.count == 0 {
        return;
    }
    let (icon_uv, iw, ih) = match lookups.item_icon(item.kind) {
        Some(v) => v,
        None => match lookups.plain_tex(item.kind) {
            Some(t) => (t.uv, 16.0, 16.0),
            None => return,
        },
    };
    let w = iw * scale;
    let h = ih * scale;
    r.push_ui_sprite(
        [x + (slot - w) * 0.5, y + (slot - h) * 0.5],
        [w, h],
        icon_uv,
        [1.0; 4],
    );
    let count = format!("{}", item.count);
    let fs = 0.75 * scale;
    let tw = hud_item_text_width(r, lookups, &count, fs);
    let tx = x + slot - tw - 4.0 * scale;
    let ty = y + slot - 11.0 * scale;
    hud_item_text(r, lookups, tx + 1.0, ty + 1.0, fs, [0.0, 0.0, 0.0, 1.0], &count);
    hud_item_text(r, lookups, tx, ty, fs, [1.0, 1.0, 1.0, 1.0], &count);
}

/// The hotbar digits (`FontAssets.ItemStack` in the original): grey by default,
/// white at 1.4x on the selected slot, and slot 10 is drawn as "0".
pub(super) fn draw_slot_number(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    i: usize,
    scale: f32,
    selected: bool,
) {
    let mut num = format!("{}", i + 1);
    if num == "10" {
        num = "0".into();
    }
    let (fs, dy, col) = if selected {
        (scale * 1.4, -2.0 * scale, [1.0, 1.0, 1.0, 200.0 / 255.0])
    } else {
        (scale, 0.0, [0.86, 0.86, 0.86, 1.0])
    };
    let px = x + 6.0 * scale;
    let py = y + 4.0 * scale + dy;
    hud_item_text(r, lookups, px + 1.0, py + 1.0, fs, [0.0, 0.0, 0.0, 1.0], &num);
    hud_item_text(r, lookups, px, py, fs, col, &num);
}
