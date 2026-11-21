//! The loadout buttons beside the equipment column — `Main.DrawLoadoutButtons`
//! (`Main.cs:41935-41991`). Three 32x32 buttons on the `Extra_259` sheet
//! (`Frame(3, 3, frameX, i)`: frameX 0 idle / 1 active / 2 hover-overlay,
//! frameY the loadout index), drawn at `screenWidth - 44`, stacked 36 px apart
//! from `inventoryTop`. Hovering lerps the button's loadout colour toward white
//! and shows the `UI.LoadoutN` mouse text; a left press-edge switches the set
//! (`Player.TrySwitchingLoadout`, `Player.cs:5661`). The hover state drives the
//! 12 cue through `Main.DoStatefulTickSound` (`Main.cs:3249-3259`).

use tr_render::SpriteRenderer;

use crate::assets;
use crate::ui::color;

/// `Main.cs:41954-41955`: `num5 = 32` (button size), `num6 = 4` (gap).
const BUTTON: f32 = 32.0;
const GAP: f32 = 4.0;
/// `Main.cs:41937-41946`: `num = 10`, minus one per unavailable extra-accessory
/// slot (Demon Heart / master mode). The port models neither
/// (`ui::inventory::DEMON_HEART_SLOT`/`MASTER_SLOT` are false), so `num = 8`.
const ROW_COUNT: i32 = 8;
/// `Main.cs:41017`: `Main.inventoryScale = 0.85f`.
const INVENTORY_SCALE: f32 = 0.85;

/// `Main.cs:41947-41951`: the button column's anchor rectangle —
/// `x = screenWidth - 58 + 14`, `y = inventoryTop - 2 + 2`, width 4, height
/// `num * 56 * inventoryScale`. The buttons hang off its right edge.
fn anchor_rect(screen_w: f32, base_y: f32) -> [f32; 4] {
    let x = screen_w - 58.0 + 14.0;
    let num2 = (base_y - 2.0) as i32 as f32; // `+ 0f * inventoryScale`
    let num3 = (base_y - 2.0 + (ROW_COUNT as f32 * 56.0) * INVENTORY_SCALE) as i32 as f32;
    [x, num2 + 2.0, 4.0, num3 - num2]
}

/// `Main.cs:41960`: button `i`'s rect —
/// `(rectangle.X + rectangle.Width, rectangle.Y + (num5 + num6) * i, 32, num5)`.
pub fn button_rect(screen_w: f32, base_y: f32, i: usize) -> [f32; 4] {
    let a = anchor_rect(screen_w, base_y);
    [
        a[0] + a[2],
        a[1] + (BUTTON + GAP) * i as f32,
        BUTTON,
        BUTTON,
    ]
}

/// `rectangle2.Contains(Main.MouseScreen.ToPoint())` (`Main.cs:41965`) —
/// `Rectangle.Contains` excludes the right/bottom edges.
pub fn button_hit(screen_w: f32, base_y: f32, mouse: [f32; 2]) -> Option<usize> {
    for i in 0..3 {
        let r = button_rect(screen_w, base_y, i);
        if mouse[0] >= r[0]
            && mouse[0] < r[0] + r[2]
            && mouse[1] >= r[1]
            && mouse[1] < r[1] + r[3]
        {
            return Some(i);
        }
    }
    None
}

/// `ItemSlot.ShiftHueByLoadout` (`ItemSlot.cs:2829-2832`):
/// `(hue + loadoutIndex / 8f) % 1f`.
pub fn shift_hue_by_loadout(hue: f32, loadout_index: usize) -> f32 {
    (hue + loadout_index as f32 / 8.0) % 1.0
}

/// `ItemSlot.GetLoadoutColor` (`ItemSlot.cs:2834-2837`):
/// `Main.hslToRgb(ShiftHueByLoadout(0.41f, i), 0.7f, 0.5f)`.
pub fn loadout_color(loadout_index: usize) -> [u8; 4] {
    color::hsl_to_rgb(shift_hue_by_loadout(0.41, loadout_index) as f64, 0.7, 0.5, 255)
}

/// `Utils.Frame(3, 3, frameX, frameY)` (`Utils.cs:1344`) over the `Extra_259`
/// sheet, drawn centred in `rect` at scale 1 (`Main.cs:41981-41986`).
fn draw_button_frame(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: &tr_render::SheetRect,
    rect: [f32; 4],
    frame_x: u32,
    frame_y: u32,
    tint: [f32; 4],
) {
    let w = sheet.width / 3;
    let h = sheet.height / 3;
    let x0 = sheet.x + frame_x * w;
    let y0 = sheet.y + frame_y * h;
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let uv = [
        x0 as f32 / aw,
        y0 as f32 / ah,
        (x0 + w) as f32 / aw,
        (y0 + h) as f32 / ah,
    ];
    // `rectangle2.Center - rectangle3.Size() / 2` (origin = half the frame,
    // scale 1) — the frame's native size centred in the 32x32 rect.
    let cx = rect[0] + rect[2] * 0.5;
    let cy = rect[1] + rect[3] * 0.5;
    r.push_ui_sprite(
        [cx - w as f32 * 0.5, cy - h as f32 * 0.5],
        [w as f32, h as f32],
        uv,
        tint,
    );
}

/// `Main.DrawLoadoutButtons` (`Main.cs:41935-41991`): the three buttons. The
/// base frame is drawn in `Color.White` (`:41982`); a hovered button overlays
/// frame 2 in `Main.OurFavoriteColor` (`:41985-41986`). Returns the hovered
/// index (`state`, -1 = none) for `DoStatefulTickSound` (`:41990`).
/// `PlayerInput.IgnoreMouseInterface` is not ported (always false).
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen_w: f32,
    base_y: f32,
    current_loadout: usize,
    mouse: [f32; 2],
) -> i32 {
    let Some(sheet) = lookups.extra_259 else {
        return -1;
    };
    let hovered = button_hit(screen_w, base_y, mouse);
    for i in 0..3usize {
        let rect = button_rect(screen_w, base_y, i);
        let frame_x = if i == current_loadout { 1 } else { 0 };
        draw_button_frame(r, lookups, &sheet, rect, frame_x, i as u32, [1.0; 4]);
        if hovered == Some(i) {
            // `Main.OurFavoriteColor = (255, 231, 69)` (`Main.cs:66748`).
            draw_button_frame(
                r,
                lookups,
                &sheet,
                rect,
                2,
                i as u32,
                color::tint([255, 231, 69, 255]),
            );
        }
    }
    hovered.map_or(-1, |i| i as i32)
}

/// The hovered button's `UI.LoadoutN` mouse text (`Main.cs:41972`,
/// `Language.GetTextValue("UI.Loadout" + (i + 1))`).
pub fn hover_label(i: usize) -> String {
    crate::lang::text(&format!("UI.Loadout{}", i + 1))
}

/// `ItemSlot.canLoadoutShareAt` (`ItemSlot.cs:395-399`): the equip contexts
/// 8/9/10/11/12 (armour, vanity, accessories, vanity accessories, dye) carry
/// the loadout-share flag in place of the grid favourite.
pub fn can_loadout_share_at(context: i32) -> bool {
    matches!(context, 8 | 9 | 10 | 11 | 12)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// W4-g55 — the button geometry at 1280x720, `inventoryTop = 174`:
    /// `x = 1280 - 44 = 1236`, `y = 174 + 36*i`, 32x32 (`Main.cs:41947-41960`).
    #[test]
    fn w4_g55_the_button_rects_match_the_csharp_layout() {
        let r0 = button_rect(1280.0, 174.0, 0);
        assert_eq!(r0, [1236.0, 174.0, 32.0, 32.0]);
        let r1 = button_rect(1280.0, 174.0, 1);
        assert_eq!(r1, [1236.0, 210.0, 32.0, 32.0]);
        let r2 = button_rect(1280.0, 174.0, 2);
        assert_eq!(r2, [1236.0, 246.0, 32.0, 32.0]);
    }

    /// W4-g55 — the hit test picks the button under the pointer and excludes
    /// the right/bottom edges (`Rectangle.Contains`).
    #[test]
    fn w4_g55_the_hit_test_picks_the_hovered_button() {
        assert_eq!(button_hit(1280.0, 174.0, [1240.0, 180.0]), Some(0));
        assert_eq!(button_hit(1280.0, 174.0, [1240.0, 215.0]), Some(1));
        assert_eq!(button_hit(1280.0, 174.0, [1240.0, 250.0]), Some(2));
        assert_eq!(button_hit(1280.0, 174.0, [1200.0, 180.0]), None);
        // The right/bottom edges are excluded.
        assert_eq!(button_hit(1280.0, 174.0, [1268.0, 180.0]), None);
        assert_eq!(button_hit(1280.0, 174.0, [1240.0, 206.0]), None);
    }

    /// W4-g55 — `GetLoadoutColor` shifts the 0.41 hue by `i/8`
    /// (`ItemSlot.cs:2829-2837`); the three buttons are distinct colours.
    #[test]
    fn w4_g55_the_loadout_colours_shift_the_hue() {
        assert_eq!(shift_hue_by_loadout(0.41, 0), 0.41);
        assert!((shift_hue_by_loadout(0.41, 1) - 0.535).abs() < 1e-6);
        assert!((shift_hue_by_loadout(0.41, 2) - 0.66).abs() < 1e-6);
        let c0 = loadout_color(0);
        let c1 = loadout_color(1);
        let c2 = loadout_color(2);
        assert_ne!(c0, c1);
        assert_ne!(c1, c2);
        assert_eq!(c0[3], 255);
    }
}
