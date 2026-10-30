//! In-game HUD: hotbar / open inventory, selected-item name, the item carried on the
//! cursor, hearts and breath, and the boss bar.

pub mod slot;
pub mod text;

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::ItemSlot;
use crate::assets;
use crate::player::{BREATH_MAX, Player, WIDTH as PLAYER_WIDTH};
use crate::{HOTBAR, INV};

/// Vanilla HUD layout: hotbar top-left (or the open 10x5 inventory grid), hearts
/// top-right, breath bubbles above the player and the boss bar at the bottom centre
/// (`GUIHotbarDrawInner`, `Main.DrawInventory`, `ClassicPlayerResourcesDisplaySet`,
/// `BigProgressBarHelper`).
#[allow(clippy::too_many_arguments)]
pub fn draw_hud(
    r: &mut SpriteRenderer,
    camera: Vec2,
    zoom: f32,
    screen: Vec2,
    lookups: &assets::Lookups,
    inventory: &[ItemSlot; INV],
    selected: usize,
    hotbar_scales: &[f32; HOTBAR],
    player: Option<&Player>,
    boss: Option<(i32, i32)>,
    inventory_open: bool,
    mouse_item: &ItemSlot,
    mouse_pos: Vec2,
) {
    // `ItemSlot.Draw` / `Main.DrawInventory`: with the inventory open the hotbar is
    // row 0 of a 10x5 grid of 56 px cells at (20,20) at `inventoryScale = 0.85`
    // (context 0: grey `InventoryBack`, hotbar row `InventoryBack9`); closed, the ten
    // slots animate between 0.75 and 1.0 (context 13: grey back at 200/255, the
    // selected one yellow `InventoryBack14` at full white).
    if inventory_open {
        let inv_scale = 0.85f32;
        if let Some(back) = lookups.hud_uv(lookups.ui_back.as_ref()) {
            let hot_back = lookups.hud_uv(lookups.ui_back_hotbar.as_ref()).unwrap_or(back);
            for j in 0..5usize {
                for i in 0..10usize {
                    let idx = i + j * 10;
                    let x = 20.0 + i as f32 * 56.0 * inv_scale;
                    let y = 20.0 + j as f32 * 56.0 * inv_scale;
                    let slot = 52.0 * inv_scale;
                    let uv = if j == 0 { hot_back } else { back };
                    let c = 220.0 / 255.0;
                    r.push_ui_sprite([x, y], [slot, slot], uv, [c, c, c, c]);
                    let item = &inventory[idx];
                    slot::draw_slot_item(r, lookups, item, x, y, slot, inv_scale);
                    if j == 0 {
                        slot::draw_slot_number(
                            r,
                            lookups,
                            x,
                            y,
                            i,
                            inv_scale,
                            idx == selected.min(HOTBAR - 1),
                        );
                    }
                }
            }
        }
    } else if let Some(back) = lookups.hud_uv(lookups.ui_back.as_ref()) {
        let sel_back = lookups.hud_uv(lookups.ui_back_sel.as_ref()).unwrap_or(back);
        let mut x = 20.0;
        for i in 0..HOTBAR {
            let scale = hotbar_scales[i].clamp(0.75, 1.0);
            let slot = 52.0 * scale;
            let y = 20.0 + 22.0 * (1.0 - scale);
            let (uv, c) = if i == selected {
                (sel_back, 255.0 / 255.0)
            } else {
                (back, 200.0 / 255.0)
            };
            r.push_ui_sprite([x, y], [slot, slot], uv, [c, c, c, c]);
            slot::draw_slot_item(r, lookups, &inventory[i], x, y, slot, scale);
            slot::draw_slot_number(r, lookups, x, y, i, scale, i == selected);
            x += slot + 4.0;
        }
    }
    // Selected item name, centred above the bar like the original.
    let item = &inventory[selected.min(HOTBAR - 1)];
    if item.count > 0 && !inventory_open {
        let label = item.name;
        let tw = text::hud_text_width(r, lookups, label, 1.0);
        let tx = 236.0 - tw * 0.5;
        text::hud_text(r, lookups, tx + 1.0, 1.0, 1.0, [0.0, 0.0, 0.0, 1.0], label);
        text::hud_text(r, lookups, tx, 0.0, 1.0, [1.0, 0.95, 0.85, 1.0], label);
    }

    // `Main.mouseItem`: the stack carried on the cursor, centred on it.
    if mouse_item.count > 0 {
        slot::draw_slot_item(
            r,
            lookups,
            mouse_item,
            mouse_pos.x - 26.0,
            mouse_pos.y - 26.0,
            52.0,
            1.0,
        );
    }

    // Hearts: 20 HP per heart, anchored to the right like the classic display.
    if let Some(p) = player {
        if let Some(heart) = lookups.hud_uv(lookups.ui_heart.as_ref()) {
            let hearts = ((p.max_hp as f32) / 20.0).ceil().max(1.0) as i32;
            let x0 = screen.x - 300.0;
            for i in 1..=hearts {
                let ratio = ((p.hp as f32 - (i - 1) as f32 * 20.0) / 20.0).clamp(0.0, 1.0);
                let (grey, scale) = if ratio >= 1.0 {
                    (255.0f32, 1.0f32)
                } else {
                    (
                        (30.0 + 225.0 * ratio).max(30.0),
                        (ratio / 4.0 + 0.75).max(0.75),
                    )
                };
                let size = 22.0 * scale;
                let x = x0 + 26.0 * (i - 1) as f32;
                let y = 32.0 + (22.0 - size) * 0.5;
                let t = grey / 255.0;
                r.push_ui_sprite([x, y], [size, size], heart, [t, t, t, 1.0]);
            }
            let label = "Life";
            let value = format!("{}/{}", p.hp, p.max_hp);
            let center = x0 + 13.0 * hearts as f32;
            let lw = text::hud_text_width(r, lookups, label, 1.0);
            text::hud_text(r, lookups, center - 3.0 - lw + 1.0, 7.0, 1.0, [0.0, 0.0, 0.0, 1.0], label);
            text::hud_text(r, lookups, center - 3.0 - lw, 6.0, 1.0, [1.0, 1.0, 1.0, 1.0], label);
            let vx = center + 3.0;
            text::hud_text(r, lookups, vx + 1.0, 7.0, 1.0, [0.0, 0.0, 0.0, 1.0], &value);
            text::hud_text(r, lookups, vx, 6.0, 1.0, [1.0, 1.0, 1.0, 1.0], &value);

            // Breath bubbles above the player's head while submerged.
            if p.breath < BREATH_MAX {
                if let Some(bubble) = lookups.hud_uv(lookups.ui_bubble.as_ref()) {
                    let psx = (p.x + PLAYER_WIDTH * 0.5 - camera.x) * zoom + screen.x * 0.5;
                    let psy = (p.y - camera.y) * zoom + screen.y * 0.5;
                    let bx = psx - 125.0;
                    let by = psy - 100.0;
                    for i in 1..=10 {
                        let ratio =
                            ((p.breath as f32 - (i - 1) as f32 * 20.0) / 20.0).clamp(0.0, 1.0);
                        let (grey, scale) = if p.breath >= i * 20 {
                            (255.0f32, 1.0f32)
                        } else {
                            (
                                (30.0 + 225.0 * ratio).max(30.0),
                                (ratio / 4.0 + 0.75).max(0.75),
                            )
                        };
                        let size = 22.0 * scale;
                        let x = bx + 26.0 * (i - 1) as f32;
                        let y = by + (22.0 - size) * 0.5;
                        let t = grey / 255.0;
                        r.push_ui_sprite([x, y], [size, size], bubble, [t, t, t, 1.0]);
                    }
                }
            }
        }
    }

    // Boss bar: `BigProgressBarHelper.DrawFancyBar` (456x22 bar at the bottom).
    if let Some((hp, max_hp)) = boss {
        let frac = (hp as f32 / max_hp.max(1) as f32).clamp(0.0, 1.0);
        let center = [screen.x * 0.5 - 228.0, screen.y - 61.0];
        let top_left = [center[0] - 32.0, center[1] - 24.0];
        if let (Some(back), Some(fill), Some(cap), Some(overlay)) = (
            lookups.boss_bar_uv(3, 0, 0, 516, 58),
            lookups.boss_bar_uv(2, 32, 24, 2, 22),
            lookups.boss_bar_uv(1, 32, 24, 2, 22),
            lookups.boss_bar_uv(0, 0, 0, 516, 58),
        ) {
            r.push_ui_sprite(top_left, [516.0, 58.0], back, [1.0, 1.0, 1.0, 0.2]);
            let width = ((456.0 * frac) as i32) & !1;
            if width > 0 {
                r.push_ui_sprite(
                    center,
                    [width as f32, 22.0],
                    fill,
                    [1.0, 1.0, 1.0, 1.0],
                );
                r.push_ui_sprite(
                    [center[0] + width as f32 - 2.0, center[1]],
                    [2.0, 22.0],
                    cap,
                    [1.0; 4],
                );
            }
            r.push_ui_sprite(top_left, [516.0, 58.0], overlay, [1.0; 4]);
            if let Some(icon) = lookups.hud_uv(lookups.ui_boss_head.as_ref()) {
                r.push_ui_sprite(
                    [top_left[0] + 2.0, top_left[1] + 19.0],
                    [30.0, 30.0],
                    icon,
                    [1.0; 4],
                );
            }
            let label = format!("{hp}/{max_hp}");
            let tw = r.text_width(&label, 1.0);
            let text_x = center[0] + 228.0 - tw * 0.5;
            r.push_text(
                text_x + 1.0,
                center[1] + 8.0,
                1.0,
                [0.0, 0.0, 0.0, 1.0],
                &label,
            );
            r.push_text(
                text_x,
                center[1] + 7.0,
                1.0,
                [1.0, 1.0, 1.0, 1.0],
                &label,
            );
        }
    }
}
