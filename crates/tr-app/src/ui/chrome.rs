//! Inventory chrome: the equipment page tabs, the defence counter, the guide/craft
//! text, the PVP/team icons and the hotbar-lock icon.
//!
//! Ported from `Main.DrawPageIcons` (`Main.cs:42787`), `Main.DrawDefenseCounter`
//! (`Main.cs:42108`), `Main.DrawGuideCraftText` (`Main.cs:42184`),
//! `Main.DrawPVPIcons` (`Main.cs:42654`) and `Main.DrawHotbarLockIcon`
//! (`Main.cs:42479`). Every literal mirrors the cited line; `Frame(cols, rows,
//! frameX, frameY)` follows `Utils.Frame` (`Utils.cs:1344`): integer cell sizes
//! `tex.Width / horizontalFrames`, `tex.Height / verticalFrames`.

use glam::Vec2;
use tr_render::{SheetRect, SpriteRenderer};

use super::text::{hud_text, hud_text_width};
use crate::assets;

/// `Main.OurFavoriteColor = new Color(255, 231, 69)` (`Main.cs:66748`): the glow
/// behind a hovered page tab (`Main.cs:42799,42811,42823`).
const FAVORITE: [f32; 4] = [255.0 / 255.0, 231.0 / 255.0, 69.0 / 255.0, 1.0];
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// A sheet sub-frame: its atlas UV plus the frame's native (unscaled) size.
#[derive(Clone, Copy)]
struct Frame {
    uv: [f32; 4],
    w: f32,
    h: f32,
}

/// `Utils.Frame(horizontalFrames, verticalFrames, frameX, frameY)` (`Utils.cs:1344`):
/// `num = tex.Width / horizontalFrames`, `num2 = tex.Height / verticalFrames`,
/// rect = `(num * frameX, num2 * frameY, num, num2)` (integer division).
fn frame(rect: &SheetRect, atlas: (u32, u32), cols: u32, rows: u32, fx: u32, fy: u32) -> Frame {
    let w = rect.width / cols.max(1);
    let h = rect.height / rows.max(1);
    let x0 = rect.x + fx * w;
    let y0 = rect.y + fy * h;
    Frame {
        uv: [
            x0 as f32 / atlas.0 as f32,
            y0 as f32 / atlas.1 as f32,
            (x0 + w) as f32 / atlas.0 as f32,
            (y0 + h) as f32 / atlas.1 as f32,
        ],
        w: w as f32,
        h: h as f32,
    }
}

/// One `MouseText` line height (`DynamicSpriteFont.MeasureString(...).Y`); the
/// built-in bitmap fallback draws 8 px glyphs (matches `ui::tooltip::line_height`).
fn line_height(lookups: &assets::Lookups) -> f32 {
    match &lookups.ui_font {
        Some(f) => f.line_spacing,
        None => 8.0,
    }
}

// ---------------------------------------------------------------------------
// Page icons — `Main.DrawPageIcons` (`Main.cs:42787-42868`)
// ---------------------------------------------------------------------------

/// The three tab anchors. Tab order is 2 (misc/equipment), 1 (housing),
/// 3 (camera), matching the draw/click order in the original.
const PAGE_TABS: [i32; 3] = [2, 1, 3];

/// `Main.cs:42790-42803`: `vector = (screenWidth - 162, yPos); vector.X += 82f`
/// → the first tab is at `screenWidth - 80`, each following tab `- 48`.
fn page_tab_x(screen_w: f32, index: usize) -> f32 {
    (screen_w - 162.0 + 82.0) - index as f32 * 48.0
}

/// `Main.cs:42792,42804,42816`: the `UI/DisplaySlots_*` sheet for a tab, honouring
/// the currently selected equipment page.
fn page_icon_index(tab: i32, equip_page: i32) -> usize {
    match tab {
        2 => {
            if equip_page == 2 {
                3
            } else {
                2
            }
        } // EquipPage[(EquipPage == 2) ? 3 : 2]
        1 => {
            if equip_page == 1 {
                5
            } else {
                4
            }
        } // [(EquipPage == 1) ? 5 : 4]
        _ => {
            if equip_page == 3 {
                10
            } else {
                8
            }
        } // [(EquipPage == 3) ? 10 : 8]
    }
}

/// The hover glow sheet for a tab (`DisplaySlots_6`/`_7`/`_9`).
fn page_highlight_index(tab: i32) -> usize {
    match tab {
        2 => 6,
        1 => 7,
        _ => 9,
    }
}

/// `Main.DrawPageIcons` (`Main.cs:42787`): the three equipment-page tabs at
/// `screenWidth - 80 / -128 / -176`, `y = yPos`, scale `0.9`. Returns the hovered
/// tab number (1/2/3) or `None`.
///
/// Deviation from the C# original: the `Main.mouseItem.stack < 1 (|| dye > 0)`
/// guards (`Main.cs:42793,42805,42817`) need the cursor stack, which is not part of
/// this signature, so the hit test is purely geometric. `Main.CaptureModeDisabled`
/// (`Main.cs:42821-42825`, which reddens tab 3) is likewise not modelled — tab 3 is
/// drawn white. `mouse` is an added trailing parameter (UI coordinates).
pub fn draw_page_icons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    equip_page: i32,
    y_pos: f32,
    mouse: Vec2,
) -> Option<i32> {
    let mut hovered = None;
    for (i, &tab) in PAGE_TABS.iter().enumerate() {
        let x = page_tab_x(screen.x, i);
        let Some(rect) = lookups.page_icons[page_icon_index(tab, equip_page)] else {
            continue;
        };
        // `Collision.CheckAABBvAABBCollision(vector, value.Size(), mouse, Vec2.One)`
        // (`Main.cs:42793`): the *native* (unscaled) tab rectangle at (x, yPos).
        let hit = mouse.x >= x
            && mouse.x < x + rect.width as f32
            && mouse.y >= y_pos
            && mouse.y < y_pos + rect.height as f32;
        if hit {
            hovered = Some(tab);
        }
        // Highlight first, then the icon (`Main.cs:42799-42801` etc).
        if hit {
            if let Some(hl) = lookups.page_icons[page_highlight_index(tab)] {
                // Origin `new Vector2(2f)` for tabs 2/1, `Vector2.Zero` for tab 3.
                let o = if tab == 3 { 0.0 } else { 2.0 };
                let f = frame(&hl, lookups.atlas_size, 1, 1, 0, 0);
                r.push_ui_sprite(
                    [x - o * 0.9, y_pos - o * 0.9],
                    [f.w * 0.9, f.h * 0.9],
                    f.uv,
                    FAVORITE,
                );
            }
        }
        let f = frame(&rect, lookups.atlas_size, 1, 1, 0, 0);
        r.push_ui_sprite([x, y_pos], [f.w * 0.9, f.h * 0.9], f.uv, WHITE);
    }
    hovered
}

// ---------------------------------------------------------------------------
// Defence counter — `Main.DrawDefenseCounter` (`Main.cs:42108-42182`)
// ---------------------------------------------------------------------------

/// `Main.cs:42110`: the shield's top-left anchor is the inventory origin shifted by
/// `- 10 - 47 - 47 - 14`, and vertically by half the `Inventory_Back` sheet height.
fn defense_shield_pos(x: f32, y: f32, back_height: f32) -> Vec2 {
    Vec2::new(x - 10.0 - 47.0 - 47.0 - 14.0, y + back_height * 0.5)
}

/// `Main.DrawDefenseCounter` (`Main.cs:42108`): the `Extra_58` shield
/// `Frame(3, 2, 0)`, centred on its origin at `scale`, with the defence number
/// centred over it.
///
/// The difficulty tint and the `Frame(3, 2, num, 1)` overlay (`Main.cs:42126-42154`)
/// and the `"<n> Defense"` tooltip + `statDefense * difficulty` damage-reduction line
/// (`Main.cs:42160-42180`) are out of scope: the tooltip goes through the tooltip
/// layer, and `Main`'s expert/master/legendary state is not plumbed in here.
pub fn draw_defense_counter(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    defense: i32,
    scale: f32,
) {
    // `TextureAssets.InventoryBack.Height()` (`Main.cs:42110`); the sheet is 56 px.
    let back_h = lookups
        .ui_back
        .as_ref()
        .map(|s| s.height as f32)
        .unwrap_or(56.0);
    let pos = defense_shield_pos(x, y, back_h);
    if let Some(rect) = lookups.ui_defense_shield.as_ref() {
        let f = frame(rect, lookups.atlas_size, 3, 2, 0, 0);
        // Origin `rectangle.Size() / 2f`, scale `Main.inventoryScale` (`Main.cs:42150-42151`).
        r.push_ui_sprite(
            [pos.x - f.w * 0.5 * scale, pos.y - f.h * 0.5 * scale],
            [f.w * scale, f.h * scale],
            f.uv,
            WHITE,
        );
    }
    // `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333`): the text
    // is centred via `position - MeasureString * 0.5 * scale + (0, 2) * scale`
    // (`Main.cs:42157-42159`), with a 4-direction black shadow at spread 2.
    let text = defense.to_string();
    let tw = hud_text_width(r, lookups, &text, scale);
    let tx = pos.x - tw * 0.5 * scale;
    let ty = pos.y - line_height(lookups) * 0.5 * scale + 2.0 * scale;
    for (ox, oy) in [(-2.0f32, 0.0f32), (2.0, 0.0), (0.0, -2.0), (0.0, 2.0)] {
        hud_text(
            r,
            lookups,
            tx + ox,
            ty + oy,
            scale,
            [0.0, 0.0, 0.0, 1.0],
            &text,
        );
    }
    hud_text(r, lookups, tx, ty, scale, WHITE, &text);
}

// ---------------------------------------------------------------------------
// Guide / craft text — `Main.DrawGuideCraftText` (`Main.cs:42184-42199`)
// ---------------------------------------------------------------------------

/// `Main.cs:42187`: the text origin is `(inventoryX + 50, inventoryY + 12)`.
/// `Main.cs:42196` then shifts it up by 14 for the two-line case, so the first line
/// lands at `y - 2` and the second at `y + 24`.
fn guide_origin(x: f32, y: f32) -> Vec2 {
    Vec2::new(x + 50.0, y + 12.0)
}
fn guide_line1_y(y: f32) -> f32 {
    guide_origin(0.0, y).y - 14.0
}
fn guide_line2_y(y: f32) -> f32 {
    guide_line1_y(y) + 26.0
}

/// `Main.DrawGuideCraftText` (`Main.cs:42184`): `Lang.inter[24]` "Place a material
/// here" when no recipe material is selected, otherwise two lines.
///
/// The origin is drawn with `Main.mouseTextColor` as an RGBA grey
/// (`Main.cs:42186`); without a pulse parameter this port uses white and leaves the
/// tint to the caller's render state. The recipe-requirements string is spelled out
/// as `Lang.inter[22] + " " + Lang.inter[23]` (`Main.cs:42232-42237`) with the only
/// `None` case P2 needs.
pub fn draw_guide_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    guide_name: Option<&str>,
) {
    let ox = guide_origin(x, y).x;
    match guide_name {
        None => hud_text(
            r,
            lookups,
            ox,
            guide_origin(x, y).y,
            1.0,
            WHITE,
            "Place a material here", // Lang.inter[24]
        ),
        Some(name) => {
            // Lang.inter[21] + " " + guideItem.Name (`Main.cs:42194`).
            let line1 = format!("Showing recipes that use {name}");
            hud_text(r, lookups, ox, guide_line1_y(y), 1.0, WHITE, &line1);
            // Lang.inter[22] + " " + Lang.inter[23] (`Main.cs:42237`).
            hud_text(
                r,
                lookups,
                ox,
                guide_line2_y(y),
                1.0,
                WHITE,
                "Required objects: None",
            );
        }
    }
}

// ---------------------------------------------------------------------------
// PVP / team icons — `Main.DrawPVPIcons` (`Main.cs:42654-42708`)
// ---------------------------------------------------------------------------

/// `Main.cs:42665-42670`: `num = (int)(52 * 0.6) = 31`, the shared x anchor
/// `707 - num * 4 + screenWidth - 800`, plus `num + num / 2` on the misc-equip page.
fn pvp_anchor_x(screen_w: f32, equip_page: i32) -> i32 {
    let num = (52.0 * 0.6) as i32; // 31
    let mut num2 = 707 - num * 4 + screen_w as i32 - 800;
    if equip_page == 2 {
        num2 += num + num / 2;
    }
    num2
}

/// `Main.cs:42667,42700`: the PVP toggle's top-left `(num2 - 10, num3)`.
fn pvp_icon_xy(screen_w: f32, equip_page: i32, m_h: f32) -> (f32, f32) {
    let num = (52.0 * 0.6) as i32;
    let num3 = 114 + m_h as i32 + num * 2 + num / 2 - 12;
    ((pvp_anchor_x(screen_w, equip_page) - 10) as f32, num3 as f32)
}

/// `Main.cs:42709-42719`: the six team icons, `num3 += 60; num2 -= 10`, then
/// `(num2 + i % 2 * 20, num3 + i / 2 * 20)`.
fn pvp_team_xy(screen_w: f32, equip_page: i32, m_h: f32, i: i32) -> (f32, f32) {
    let (x, y) = pvp_icon_xy(screen_w, equip_page, m_h);
    (x + (i % 2) as f32 * 20.0, y + 60.0 + (i / 2) as f32 * 20.0)
}

/// `Main.DrawPVPIcons` (`Main.cs:42654`): the PVP toggle `PVP_0.Frame(2, 7)` and the
/// six team squares `PVP_1.Frame(6)`, both drawn at scale 1 (`Vector2.One`, not the
/// `inventoryScale = 0.6` used only for the position maths).
///
/// Deviations: the original gates on `ShouldPVPDraw`/`ShouldTeamSelectDraw` and
/// skips while `hidePVPAndTeamIcons` (`Main.cs:42656-42662`); this port draws both.
/// The PVP frame is `(hostile, team)` = `(false, 0)` — the cell shown when the player
/// is not hostile and on team 0 — since neither is in the signature. `num4`
/// (`teamCooldown`, `Main.cs:42672`) dims the icon to `White * 0.25` during the
/// cooldown; drawn at `White` here. `draw_pvp_icons` does not mutate a global
/// `inventoryScale`.
pub fn draw_pvp_icons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    equip_page: i32,
    m_h: f32,
) {
    let (x, y) = pvp_icon_xy(screen.x, equip_page, m_h);
    if let Some(rect) = lookups.pvp[0].as_ref() {
        let f = frame(rect, lookups.atlas_size, 2, 7, 0, 0);
        r.push_ui_sprite([x, y], [f.w, f.h], f.uv, WHITE);
    }
    if let Some(rect) = lookups.pvp[1].as_ref() {
        for i in 0..6i32 {
            let (tx, ty) = pvp_team_xy(screen.x, equip_page, m_h, i);
            let f = frame(rect, lookups.atlas_size, 6, 1, i as u32, 0);
            r.push_ui_sprite([tx, ty], [f.w, f.h], f.uv, WHITE);
        }
    }
}

// ---------------------------------------------------------------------------
// Hotbar lock icon — `Main.DrawHotbarLockIcon` (`Main.cs:42479-42516`)
// ---------------------------------------------------------------------------

/// `Main.cs:42481-42486`: `num = 21 + pivotTopLeftY` (here `0`), or the pivot
/// itself when the side tools are pushed up.
fn hotbar_lock_y(push_side_tools_up: bool) -> f32 {
    if push_side_tools_up {
        0.0
    } else {
        21.0
    }
}

/// `Main.DrawHotbarLockIcon(0, 0, pushSideToolsUp)` (`Main.cs:42479`).
///
/// **Documented no-op.** The original draws `TextureAssets.HbLock[hbLocked ? 0 : 1]`
/// = `Images/Lock_0` / `Images/Lock_1` (`AssetInitializer.cs:475-476`), which are
/// present on disk but are not in `assets::Lookups` (Task 1's whitelist), so there is
/// no sheet to draw here. Adding one is outside this task's single-file scope. The
/// geometry (`(0, hotbar_lock_y)`, scale 0.9) is kept and tested for the caller.
pub fn draw_hotbar_lock(_r: &mut SpriteRenderer, _lookups: &assets::Lookups, push_side_tools_up: bool) {
    let _ = hotbar_lock_y(push_side_tools_up);
}

// ---------------------------------------------------------------------------
// Map-style icon row — `Main.DrawInventory` (`Main.cs:41055-41126`)
// ---------------------------------------------------------------------------

/// `Main.cs:41087`: the hover/click test is `32` px wide and `30` px tall (the
/// C# literals), inclusive on every edge (`mouseX >= num16 && mouseX <= num16
/// + 32 && mouseY >= num17 && mouseY <= num17 + 30`).
pub const MAP_ICON_HIT: [f32; 2] = [32.0, 30.0];

/// `Main.cs:41070-41087`: icon `k`'s top-left. At 1600x900 (>= 940 width):
/// `(screenWidth - 440 + k*32 - num14, 40)` with `num14 = 0` (`:41070`).
/// The `< 940` branch (`:41061-41068,41076-41080`) stacks vertically at
/// `(screenWidth - 40, -200 + k*32)`, kept for fidelity.
pub fn map_icon_rect(k: usize, screen: Vec2) -> [f32; 2] {
    // `if (Main.screenWidth < 940) flag = true` (`:41061-41064`).
    map_icon_rect_flag(k, screen, screen.x < 940.0)
}

/// The branched form, with `flag` the source's local (`:41058,41063`), so the
/// kept-dead `< 940` geometry stays testable at the 1600x900 reference frame.
fn map_icon_rect_flag(k: usize, screen: Vec2, flag: bool) -> [f32; 2] {
    // `int num12 = num3 - 440; int num13 = 40 + num4` (`:41059-41060`, `num3 =
    // screenWidth`, `num4 = 0`); the flag branch rewrites both
    // (`num12 = num5 - 40; num13 = num6 - 200`, `:41067-41068`, `num5 =
    // screenWidth`, `num6 = 0`).
    let (num12, num13) = if flag {
        (screen.x - 40.0, -200.0)
    } else {
        (screen.x - 440.0, 40.0)
    };
    // `int num14 = 0` (`:41070`): never reassigned in the loop (`:41071-41124`).
    let num14 = 0.0;
    if flag {
        // `num16 = num12; num17 = num13 + k * 32 - num14` (`:41078-41079`).
        [num12, num13 + k as f32 * 32.0 - num14]
    } else {
        // `num16 = num12 + k * 32 - num14; num17 = num13` (`:41074-41075`).
        [num12 + k as f32 * 32.0 - num14, num13]
    }
}

/// The `Color(num15, num15, num15, num15)` tint and the `MapIcon[num18]` sheet
/// index of one icon (`Main.cs:41073-41090,41123`): `120` base, `200` when the
/// icon is the active style (`k > 0 && Main.mapStyle == k - 1`,
/// `:41083-41086` — `k = 0` is never "active"), `255` + the `Map_{k+4}` sheet
/// while hovered (`:41087-41090`). The sheet index indexes `Lookups::map_icons`
/// (`Images/Map_0..7`, `AssetInitializer.cs:386-389`).
pub fn map_icon_appearance(k: usize, map_style: i32, hovered: bool) -> ([f32; 4], usize) {
    // `int num18 = k` (`:41081`); hovered adds 4 (`num18 += 4`, `:41090`).
    let sheet = if hovered { k + 4 } else { k };
    // `num15 = 120` (`:41082`), `num15 = 200` (`:41085`), `num15 = 255`
    // (`:41089`).
    let byte = if hovered {
        255
    } else if k > 0 && map_style == k as i32 - 1 {
        200
    } else {
        120
    };
    let c = byte as f32 / 255.0;
    ([c, c, c, c], sheet)
}

/// `Main.DrawInventory`'s row (`Main.cs:41055-41126`): the four `MapIcon[k]`
/// sheets (fullscreen map, map off, the minimap, the overlay) at
/// `map_icon_rect`, tinted per `map_icon_appearance`, drawn with the full-frame
/// source rect at scale 1 (`:41123`). `map_style` is `Main.mapStyle`.
///
/// Deviations from the C# original: the whole row sits behind
/// `if (Main.mapEnabled)` (`:41056`), which this signature cannot see — the
/// port draws it whenever the inventory is open (the clicks, wired in
/// `main.rs`, are still gated on the map being enabled); the hover's
/// `Main.DoStatefulTickSound` (`:41126`) and the clicks' `PlaySound(10)/(12)`
/// (`:41100,41109-41119`) are not ported — no UI cue is loaded
/// (`audio.rs:22-31`), consistent with the silent menus.
pub fn draw_map_style_icons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    map_style: i32,
    mouse_pos: Vec2,
) {
    for k in 0..4usize {
        let pos = map_icon_rect(k, screen);
        // `Main.cs:41087`: inclusive on all four edges.
        let hovered = mouse_pos.x >= pos[0]
            && mouse_pos.x <= pos[0] + MAP_ICON_HIT[0]
            && mouse_pos.y >= pos[1]
            && mouse_pos.y <= pos[1] + MAP_ICON_HIT[1];
        let (tint, sheet) = map_icon_appearance(k, map_style, hovered);
        let Some(rect) = lookups.map_icons.get(sheet).and_then(|s| s.as_ref()) else {
            continue; // missing sheets draw nothing (`Lookups::default()`)
        };
        let Some(uv) = lookups.hud_uv(Some(rect)) else {
            continue;
        };
        // `spriteBatch.Draw(..., new Vector2(num16, num17), Rectangle(0, 0,
        // Width, Height), Color(num15, ...))` (`:41123`): scale 1, whole sheet.
        r.push_ui_sprite(pos, [rect.width as f32, rect.height as f32], uv, tint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_tab_x_at_1600() {
        // `screenWidth - 80 / -128 / -176` (`Main.cs:42791,42803,42815`).
        assert_eq!(page_tab_x(1600.0, 0), 1520.0);
        assert_eq!(page_tab_x(1600.0, 1), 1472.0);
        assert_eq!(page_tab_x(1600.0, 2), 1424.0);
    }

    #[test]
    fn page_icon_selection() {
        assert_eq!(page_icon_index(2, 0), 2);
        assert_eq!(page_icon_index(2, 2), 3);
        assert_eq!(page_icon_index(1, 0), 4);
        assert_eq!(page_icon_index(1, 1), 5);
        assert_eq!(page_icon_index(3, 0), 8);
        assert_eq!(page_icon_index(3, 3), 10);
        assert_eq!(page_highlight_index(2), 6);
        assert_eq!(page_highlight_index(1), 7);
        assert_eq!(page_highlight_index(3), 9);
    }

    #[test]
    fn defense_shield_offset() {
        // `(inventoryX - 10 - 47 - 47 - 14, inventoryY + InventoryBack.Height()*0.5)`
        // (`Main.cs:42110`); the 56 px `Inventory_Back` sheet gives `y + 28`.
        let p = defense_shield_pos(100.0, 200.0, 56.0);
        assert_eq!(p, Vec2::new(100.0 - 118.0, 200.0 + 28.0));
    }

    #[test]
    fn guide_text_origin() {
        // Empty: `(inventoryX + 50, inventoryY + 12)` (`Main.cs:42187`).
        assert_eq!(guide_origin(73.0, 150.0), Vec2::new(123.0, 162.0));
        // Non-empty: `vector.Y -= 14` then `+ (0, 26)` (`Main.cs:42196-42198`).
        assert_eq!(guide_line1_y(150.0), 148.0);
        assert_eq!(guide_line2_y(150.0), 174.0);
    }

    #[test]
    fn pvp_icon_pos_at_1600() {
        // `num = 31`; `num2 = 707 - 124 + 1600 - 800 = 1383`; position `num2 - 10`.
        // `num3 = 114 + 0 + 62 + 15 - 12 = 179`.
        assert_eq!(pvp_anchor_x(1600.0, 0), 1383);
        assert_eq!(pvp_icon_xy(1600.0, 0, 0.0), (1373.0, 179.0));
        // Misc-equip page shifts x by `num + num/2 = 46` (`Main.cs:42670`).
        assert_eq!(pvp_icon_xy(1600.0, 2, 0.0), (1419.0, 179.0));
    }

    #[test]
    fn pvp_team_pos_at_1600() {
        // `num3 += 60; num2 -= 10` then `(num2 + i%2*20, num3 + i/2*20)`.
        assert_eq!(pvp_team_xy(1600.0, 0, 0.0, 0), (1373.0, 239.0));
        assert_eq!(pvp_team_xy(1600.0, 0, 0.0, 1), (1393.0, 239.0));
        assert_eq!(pvp_team_xy(1600.0, 0, 0.0, 5), (1393.0, 279.0));
    }

    #[test]
    fn hotbar_lock_y_both_branches() {
        // `num = 21 + pivotTopLeftY` (0), or `pivotTopLeftY` when pushed up.
        assert_eq!(hotbar_lock_y(false), 21.0);
        assert_eq!(hotbar_lock_y(true), 0.0);
    }

    #[test]
    fn map_icon_rect_at_1600x900() {
        // `Main.cs:41059-41075`: `num12 = 1600 - 440`, `num13 = 40`,
        // `num16 = num12 + k*32 - num14`, `num14 = 0`.
        assert_eq!(map_icon_rect(0, Vec2::new(1600.0, 900.0)), [1160.0, 40.0]);
        assert_eq!(map_icon_rect(1, Vec2::new(1600.0, 900.0)), [1192.0, 40.0]);
        assert_eq!(map_icon_rect(2, Vec2::new(1600.0, 900.0)), [1224.0, 40.0]);
        assert_eq!(map_icon_rect(3, Vec2::new(1600.0, 900.0)), [1256.0, 40.0]);
        // The branch selector itself (`Main.cs:41061`): a >= 940 width is the
        // horizontal row, a narrower one stacks vertically.
        assert_eq!(
            map_icon_rect(0, Vec2::new(940.0, 900.0)),
            [500.0, 40.0]
        );
        assert_eq!(
            map_icon_rect(0, Vec2::new(939.0, 900.0)),
            [899.0, -200.0]
        );
    }

    #[test]
    fn map_icon_rect_below_940_branch_kept_for_fidelity() {
        // `Main.cs:41067-41079`: `num12 = num5 - 40`, `num13 = num6 - 200`, then
        // `num16 = num12; num17 = num13 + k*32 - num14`. Dead at the port's
        // reference resolutions, tested at 1600x900 through the `flag` form.
        let screen = Vec2::new(1600.0, 900.0);
        assert_eq!(map_icon_rect_flag(0, screen, true), [1560.0, -200.0]);
        assert_eq!(map_icon_rect_flag(1, screen, true), [1560.0, -168.0]);
        assert_eq!(map_icon_rect_flag(2, screen, true), [1560.0, -136.0]);
        assert_eq!(map_icon_rect_flag(3, screen, true), [1560.0, -104.0]);
    }

    #[test]
    fn map_icon_appearance_matches_the_tint_and_sheet_table() {
        // The whole-UI P4 plan's Task 3 table, oracles `Main.cs:41073-41090`.
        let c = |b: f32| [b / 255.0; 4];
        // Hovered: 255 + the `Map_{k+4}` sheet (`:41087-41090`), any style.
        assert_eq!(map_icon_appearance(2, 0, true), (c(255.0), 6));
        // k = 0 is never "active" (`:41083`): 120 (`:41082`).
        assert_eq!(map_icon_appearance(0, 0, false), (c(120.0), 0));
        assert_eq!(map_icon_appearance(0, 2, false), (c(120.0), 0));
        // The active style icon: `k > 0 && mapStyle == k - 1` -> 200
        // (`:41083-41086`).
        assert_eq!(map_icon_appearance(1, 0, false), (c(200.0), 1));
        assert_eq!(map_icon_appearance(2, 1, false), (c(200.0), 2));
        assert_eq!(map_icon_appearance(3, 2, false), (c(200.0), 3));
        // An inactive icon stays 120.
        assert_eq!(map_icon_appearance(1, 1, false), (c(120.0), 1));
    }
}
