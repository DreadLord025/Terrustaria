//! The emote draw — `EmoteBubble.Draw` (`:209-251`) and the
//! `UIEmotesMenu` wheel (the panel + the 8-column `EmoteButton` grid).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::push_whole;
use super::text;
use crate::assets::{self, Lookups};

/// The `EmoteButton` cell — the buttons ride the sheet's 2-frame columns;
/// the 44 px grid cell keeps the 4-emote-per-row rhythm visible.
const CELL: f32 = 44.0;
/// The wheel grid's 8 columns (the sheet's 8-frame width / 2 frames).
const COLS: usize = 8;
/// The visible rows before the scroll eats the rest (the panel fits 10).
const ROWS: usize = 10;

/// The wheel's grid rect — the bestiary panel's body (the shared container
/// style, `UIEmotesMenu.cs:48-69`'s 0.875-wide panel).
fn grid_rect(screen: Vec2) -> [f32; 4] {
    let panel = super::bestiary::panel_rect(screen);
    [panel[0] + 12.0, panel[1] + 44.0, panel[2] - 24.0, panel[3] - 44.0 - 60.0]
}

/// The emote id under the cursor (the scroll-shifted grid).
pub fn card_at(screen: Vec2, mouse: Vec2, scroll: i32) -> Option<i32> {
    let rect = grid_rect(screen);
    if mouse.x < rect[0] || mouse.y < rect[1] {
        return None;
    }
    let col = ((mouse.x - rect[0]) / CELL) as usize;
    let row = ((mouse.y - rect[1]) / CELL) as usize;
    let visible_rows = ((rect[3] / CELL) as usize).min(ROWS);
    if col >= COLS || row >= visible_rows {
        return None;
    }
    let idx = row * COLS + col + scroll as usize;
    (idx < crate::emotes::EMOTE_COUNT as usize).then_some(idx as i32)
}

/// The full wheel + bubble draw (`EmoteBubble.DrawAll` + the menu state).
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    emotes: &crate::emotes::EmoteSystem,
    player_center: Vec2,
    player_dir: i32,
    mouse: Vec2,
) {
    // The bubbles (`EmoteBubble.Draw`, `:209-251`).
    if let Some(sheet) = lookups.ui_emote_sheet {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let sh = sheet.height as f32 / crate::emotes::SHEET_ROWS as f32;
        let sw = sheet.width as f32 / 8.0;
        let mut draw_frame = |col: f32, row: f32, pos: [f32; 2], flip: bool| {
            let x0 = sheet.x as f32 + sw * col;
            let y0 = sheet.y as f32 + sh * row;
            let uv = [x0 / aw, y0 / ah, (x0 + sw) / aw, (y0 + sh) / ah];
            r.push_ui_sprite(pos, [sw, sh], uv, [1.0; 4]);
            let _ = flip;
        };
        for b in &emotes.bubbles {
            // The anchor: the player's top-left + the direction offset
            // (`GetPosition`'s entity arm, `:253-261`) — the port anchors
            // the local player only.
            let pos = [
                player_center.x - 10.0 - player_dir as f32 * 15.0,
                player_center.y - 44.0,
            ];
            // The pop gate: the first/last 6 ticks draw the closed shell
            // (`:213-217`).
            let popping = b.life_time < 6 || b.life_time_start - b.life_time < 6;
            let shell_col = if popping { 0.0 } else { 1.0 };
            draw_frame(shell_col, 0.0, pos, false);
            if popping {
                continue;
            }
            // The emote sprite: column `emote*2%8 + frame`, row `1 + emote/4`
            // (`:232-236`).
            let col = (b.emote * 2) % 8 + b.frame;
            let row = 1 + b.emote / 4;
            draw_frame(col as f32, row as f32, pos, false);
        }
    }
    // The wheel (`UIEmotesMenu`): the shared panel chrome + the grid.
    if !emotes.menu_open {
        return;
    }
    let panel = super::bestiary::panel_rect(screen);
    r.push_ui_sprite(
        [panel[0], panel[1]],
        [panel[2], panel[3]],
        r.white_uv(),
        [33.0 / 255.0 * 0.8, 43.0 / 255.0 * 0.8, 79.0 / 255.0 * 0.8, 0.8],
    );
    text::hud_text(r, lookups, panel[0] + 12.0, panel[1] + 10.0, 1.0, [1.0; 4], "Emotes");
    let rect = grid_rect(screen);
    if let Some(sheet) = lookups.ui_emote_sheet {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let sh = sheet.height as f32 / crate::emotes::SHEET_ROWS as f32;
        let sw = sheet.width as f32 / 8.0;
        let visible_rows = ((rect[3] / CELL) as usize).min(ROWS);
        for row in 0..visible_rows {
            for col in 0..COLS {
                let idx = row * COLS + col + emotes.scroll as usize;
                if idx >= crate::emotes::EMOTE_COUNT as usize {
                    break;
                }
                let x = rect[0] + col as f32 * CELL + (rect[2] - COLS as f32 * CELL) * 0.5;
                let y = rect[1] + row as f32 * CELL;
                // `EmoteButton.GetFrame` (`:32-36`): the hover picks the
                // second frame (`emote%4*2 + hover`), row `emote/4 + 1`.
                let id = idx as i32;
                let hover = card_at(screen, mouse, emotes.scroll) == Some(id);
                let fcol = (id % 4) * 2 + if hover { 1 } else { 0 };
                let frow = 1 + id / 4;
                let x0 = sheet.x as f32 + sw * fcol as f32;
                let y0 = sheet.y as f32 + sh * frow as f32;
                let uv = [x0 / aw, y0 / ah, (x0 + sw) / aw, (y0 + sh) / ah];
                r.push_ui_sprite([x, y], [CELL, CELL], uv, [1.0; 4]);
            }
        }
    }
    // The Back panel (`MakeExitButton`'s shared style).
    let back = super::bestiary::back_rect(&panel);
    let style = super::cards::fat_button_style();
    let text_w = text::hud_text_width(r, lookups, "Back", style.text_scale);
    super::uistate::draw_text_panel(
        r,
        lookups,
        back,
        &style,
        "Back",
        text_w,
        super::bestiary::back_hit(screen, mouse),
    );
}
