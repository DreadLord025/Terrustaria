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

/// A whole sheet drawn at its native size (no frame grid): the accessory
/// visible/hidden ticks (`Main.cs:41455` — `spriteBatch.Draw(value3, new
/// Vector2(num41, num42), Color.White * 0.7f)`).
pub fn sheet_frame(rect: &SheetRect, atlas: (u32, u32)) -> ([f32; 4], f32, f32) {
    let f = frame(rect, atlas, 1, 1, 0, 0);
    (f.uv, f.w, f.h)
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
pub fn page_tab_x(screen_w: f32, index: usize) -> f32 {
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

/// `Main.DrawPageIcons`' hit test (`Main.cs:42793`, `:42805`, `:42817`): the
/// tab rectangle is the *native* (unscaled) `EquipPage` sheet at
/// `(page_tab_x(..), yPos)`, and each tab carries its own mouse-item gate:
///
/// * tab 2 — `Main.mouseItem.stack < 1 || Main.mouseItem.dye > 0` (`:42793`),
/// * tab 1 — `Main.mouseItem.stack < 1` (`:42805`),
/// * tab 3 — `Main.mouseItem.stack < 1` (`:42817`).
///
/// All three are unconditional on `!PlayerInput.IgnoreMouseInterface`.
///
/// `mouse_item` is `(kind, count)`; the `dye > 0` half of tab 2's gate needs
/// `Item.dye`, which `ItemDef` does not carry yet (see [`select_equip_page`]),
/// so a held dye is treated as a plain held stack — the tab is refused instead
/// of highlighted.
pub fn page_tab_hit(
    screen: Vec2,
    y_pos: f32,
    mouse: Vec2,
    mouse_item: (u16, u32),
    has_dye: bool,
    sizes: [(f32, f32); 3],
) -> Option<i32> {
    let (_, count) = mouse_item;
    for (i, &tab) in PAGE_TABS.iter().enumerate() {
        let x = page_tab_x(screen.x, i);
        // `Collision.CheckAABBvAABBCollision` against `value.Size()` at `vector`:
        // a half-open native rect (the sheet size, drawn at 0.9).
        let (w, h) = sizes[i];
        let hit = mouse.x >= x && mouse.x < x + w && mouse.y >= y_pos && mouse.y < y_pos + h;
        if !hit {
            continue;
        }
        let gate = match tab {
            2 => count == 0 || has_dye,
            _ => count == 0,
        };
        if gate {
            return Some(tab);
        }
    }
    None
}

/// The three tab sheets' native sizes, for [`page_tab_hit`]. `Main.cs:42792`,
/// `:42804`, `:42816` read `TextureAssets.EquipPage` at fixed indices for each
/// tab, and every state of one tab shares its size, so the plain form is used.
pub fn page_tab_sizes(lookups: &assets::Lookups) -> [(f32, f32); 3] {
    let pick = |i: usize| {
        lookups.page_icons[i]
            .map(|s| (s.width as f32, s.height as f32))
            .unwrap_or((0.0, 0.0))
    };
    [pick(2), pick(4), pick(8)]
}

/// The click arm of `Main.DrawPageIcons` (`Main.cs:42828-42858`): the newly
/// selected page, or `None` when the press is not a page switch.
///
/// The C# toggles: pressing the already-selected tab falls back to page 0
/// (`:42853-42857`), otherwise the pressed tab becomes the selection. Tab 3 is
/// the capture-mode entry (`:42835-42847`) and is **not** a page switch, so it
/// returns `None` here — capture mode is unported.
pub fn page_tab_click(tab: i32, selected: i32) -> Option<i32> {
    if tab == 3 {
        return None;
    }
    Some(if selected == tab { 0 } else { tab })
}

/// `ItemSlot.SelectEquipPage` (`ItemSlot.cs:4622-4652`): the page the held item
/// forces, or `-1` for "no opinion" — in which case `Main.cs:42864-42866` falls
/// back to `Main.EquipPageSelected`.
///
/// Ported arms: air (`:4625` → `-1`) and armour / accessory (`:4647-4650` → 0).
/// The remaining arms need item data `ItemDef` does not carry yet: `Item.shoot`
/// against `Main.projHook` (`:4627`), `Item.mountType` (`:4631`), the
/// `Main.vanityPet` / `Main.lightPet` buff sets (`:4635-4642`) and `Item.dye`
/// for the `EquipPageSelected == 1` arm (`:4643-4646`). All of them resolve to
/// page 2 or 0; until the generator emits those fields the misc page is reached
/// by its tab alone.
pub fn select_equip_page(kind: u16, count: u32, _selected: i32) -> i32 {
    if count == 0 || kind == 0 {
        return -1; // `item.IsAir` (`:4625`)
    }
    let d = &crate::itemdata::ITEMS[kind as usize];
    if d.head_slot != -1 || d.body_slot != -1 || d.leg_slot != -1 || d.accessory {
        return 0; // `:4647-4650`
    }
    -1
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
pub fn pvp_icon_xy(screen_w: f32, equip_page: i32, m_h: f32) -> (f32, f32) {
    let num = (52.0 * 0.6) as i32;
    let num3 = 114 + m_h as i32 + num * 2 + num / 2 - 12;
    (
        (pvp_anchor_x(screen_w, equip_page) - 10) as f32,
        num3 as f32,
    )
}

/// `Main.cs:42709-42719`: the six team icons, `num3 += 60; num2 -= 10`, then
/// `(num2 + i % 2 * 20, num3 + i / 2 * 20)`.
fn pvp_team_xy(screen_w: f32, equip_page: i32, m_h: f32, i: i32) -> (f32, f32) {
    let (x, y) = pvp_icon_xy(screen_w, equip_page, m_h);
    (x + (i % 2) as f32 * 20.0, y + 60.0 + (i / 2) as f32 * 20.0)
}

/// `Main.teamCooldownLen` (`Main.cs:66781`): 300 ticks = 5 s between PVP/team
/// changes. `Main.teamCooldown` (`Main.cs:965`) counts down once per update
/// (`Main.cs:17048-17051`).
pub const TEAM_COOLDOWN_LEN: i32 = 300;

/// `PlayerTeamID.Count` (`PlayerTeamID.cs`) — the six teams 0..=5, used as the
/// `Frame(2, 7)` row count minus the hover row (`Main.cs:42721`'s
/// `rectangle.Y = rectangle.Height * PlayerTeamID.Count` is the row-6 hover cell).
pub const PLAYER_TEAM_COUNT: i32 = 6;

/// The PVP/team icon state `Main.DrawPVPIcons` reads, passed by value so the
/// geometry stays testable without an `App` (`Main.cs:42654-42786`).
pub struct PvpIconState {
    /// `Main.netMode == 1` (`Main.cs:2909` `ShouldPVPDraw`).
    pub pvp_draw: bool,
    /// `Main.ShouldTeamSelectDraw` (`Main.cs:2911-2919`): always with the PVP
    /// icon, else only on a team-based-spawns seed.
    pub team_select_draw: bool,
    /// `Main.hidePVPAndTeamIcons` (`Main.cs:42656-42662`): the NPC-heads column
    /// overflow sets it; the draw returns early while set, and every
    /// non-`EquipPage == 1` call clears it.
    pub hide: bool,
    /// `player[myPlayer].hostile` — the `Frame(2, 7)` column (`:42664`).
    pub hostile: bool,
    /// `player[myPlayer].team` — the `Frame(2, 7)` row (`:42688`).
    pub team: i32,
    /// `Main.teamCooldown` (`:42672`): dims both icon groups to 0.25 while
    /// non-zero and gates every hover/click.
    pub team_cooldown: i32,
}

/// `Main.cs:42678-42679`: the PVP toggle's hover box, `num2 - 7 < x < num2 + 25`,
/// `num3 - 2 < y < num3 + 37` — strict on all four edges.
fn pvp_hover(x: f32, y: f32, num2: i32, num3: i32) -> bool {
    x > (num2 - 7) as f32
        && x < (num2 + 25) as f32
        && y > (num3 - 2) as f32
        && y < (num3 + 37) as f32
}

/// The C#'s per-cell team hit `r.Contains(Main.MouseScreen.ToPoint())`
/// (`Main.cs:42724`): the rect is `(num2 + i%2*20, num3 + i/2*20)` sized
/// `(width - 2, height)` — `rectangle2.Width - 2` is assigned to `r.Width`
/// after the first iteration's hover test but the `Contains` on the NEXT
/// iteration already reads the narrowed width (`:42770-42772`), so the port
/// tests against `w - 2` from the start; `Rectangle.Contains` is half-open on
/// the far edges (right/bottom exclusive).
fn team_hover(mouse: Vec2, num2: i32, num3: i32, i: i32, w: i32, h: i32) -> bool {
    let x = (num2 + i % 2 * 20) as f32;
    let y = (num3 + i / 2 * 20) as f32;
    mouse.x >= x && mouse.x < x + (w - 2) as f32 && mouse.y >= y && mouse.y < y + h as f32
}

/// The hit-tested ids for the click dispatch: `PvpToggle` when the PVP icon's
/// hover box holds the pointer, and `Team(i)` for the six team cells. The
/// callers re-check the cooldown gates (`Main.cs:42689`, `:42753`).
pub enum PvpHit {
    PvpToggle,
    Team(i32),
}

/// `Main.DrawPVPIcons`'s hit test, split from the draw so `App::handle_inventory_clicks`
/// can route a press through the same geometry (`Main.cs:42678-42682`,
/// `:42724-42729`). Returns `None` while `hide`/`pvp_draw` are off — the C#
/// only runs the tests inside those gates. `team_cell` is one `PVP_1` frame's
/// `(width, height)`; `None` falls back to the 20 px grid step the cell spacing
/// is built from (`:42770`'s `i%2*20`/`i/2*20`).
pub fn pvp_hit(
    screen: Vec2,
    equip_page: i32,
    m_h: f32,
    state: &PvpIconState,
    mouse: Vec2,
    team_cell: Option<(i32, i32)>,
) -> Option<PvpHit> {
    if state.hide || !state.pvp_draw {
        return None;
    }
    let (x, y) = pvp_icon_xy(screen.x, equip_page, m_h);
    // `Main.cs:42678`: the toggle's hover box is built from `num2` itself
    // (`num2 - 7 .. num2 + 25`), i.e. 10 px right of the sprite at `num2 - 10`.
    let num2 = x as i32 + 10;
    let num3 = y as i32;
    if pvp_hover(mouse.x, mouse.y, num2, num3) {
        return Some(PvpHit::PvpToggle);
    }
    if !state.team_select_draw {
        return None;
    }
    // `Main.cs:42718`: `num2 -= 10` before the team grid, so the cells start at
    // the sprite's own x.
    let team_num2 = num2 - 10;
    let team_num3 = num3 + 60;
    let (w, h) = team_cell.unwrap_or((20, 20));
    for i in 0..PLAYER_TEAM_COUNT {
        if team_hover(mouse, team_num2, team_num3, i, w, h) {
            return Some(PvpHit::Team(i));
        }
    }
    None
}

/// The `PVP_1` cell size from the sheet (`Frame(6)` splits the width into six).
pub fn pvp_team_cell(lookups: &assets::Lookups) -> Option<(i32, i32)> {
    lookups
        .pvp
        .get(1)
        .and_then(|r| r.as_ref())
        .map(|rect| ((rect.width / 6) as i32, rect.height as i32))
}

/// `Main.DrawPVPIcons` (`Main.cs:42654-42786`): the PVP toggle
/// `PVP_0.Frame(2, 7)` at `(hostile, team)` plus the row-6 hover cell, and the six
/// team squares `PVP_1.Frame(6)` with the `PVP_2` hover ring.
///
/// Both groups are drawn at scale 1 (`Vector2.One`); `inventoryScale = 0.6`
/// (`:42664`) only feeds the `num` position maths, so no global is mutated here.
/// `num4` (`:42672`) dims both to `White * 0.25` while `teamCooldown` runs, and a
/// hovered icon under a live cooldown draws no highlight (the flag needs
/// `teamCooldown == 0`, `:42680-42686` / `:42725-42731`).
///
/// The hover tooltip (`MouseTextHackZoom`, `UI.PvpIcon_*` / `UI.TeamIcon*`) is
/// drawn by `App`'s mouse-text pass from the same hit test, not from here.
pub fn draw_pvp_icons(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    equip_page: i32,
    m_h: f32,
    state: &PvpIconState,
    mouse: Vec2,
) {
    if state.hide {
        return;
    }
    let (x, y) = pvp_icon_xy(screen.x, equip_page, m_h);
    let num2 = x as i32 + 10;
    let num3 = y as i32;
    // `:42672`: `float num4 = ((Main.teamCooldown == 0) ? 1f : 0.25f)`.
    let dim = if state.team_cooldown == 0 {
        1.0f32
    } else {
        0.25
    };
    let tint = [dim; 4];
    if state.pvp_draw {
        let hovered = state.team_cooldown == 0 && pvp_hover(mouse.x, mouse.y, num2, num3);
        if let Some(rect) = lookups.pvp[0].as_ref() {
            let fx = u32::from(state.hostile);
            let fy = state.team.clamp(0, PLAYER_TEAM_COUNT - 1) as u32;
            let f = frame(rect, lookups.atlas_size, 2, 7, fx, fy);
            r.push_ui_sprite([x, y], [f.w, f.h], f.uv, tint);
            if hovered {
                // `:42696-42700`: the row-`PlayerTeamID.Count` cell, at full white.
                let hf = frame(rect, lookups.atlas_size, 2, 7, fx, PLAYER_TEAM_COUNT as u32);
                r.push_ui_sprite([x, y], [hf.w, hf.h], hf.uv, WHITE);
            }
        }
    }
    if !state.team_select_draw {
        return;
    }
    if let Some(rect) = lookups.pvp[1].as_ref() {
        let cw = (rect.width / 6) as i32;
        let ch = rect.height as i32;
        // `Main.cs:42718-42719`: `num2 -= 10; num3 += 60` before the grid.
        let team_num2 = num2 - 10;
        let team_num3 = num3 + 60;
        for i in 0..PLAYER_TEAM_COUNT {
            let (tx, ty) = pvp_team_xy(screen.x, equip_page, m_h, i);
            let f = frame(rect, lookups.atlas_size, 6, 1, i as u32, 0);
            // `:42770`: the hover flag needs the cooldown clear (`:42725-42729`).
            let hovered =
                state.team_cooldown == 0 && team_hover(mouse, team_num2, team_num3, i, cw, ch);
            if hovered {
                if let Some(ring) = lookups.pvp[2].as_ref() {
                    // `:42771-42773`: the ring at `Location + (-2, 0)` with the
                    // cell's narrowed width (`- 2`).
                    let rf = frame(ring, lookups.atlas_size, 1, 1, 0, 0);
                    r.push_ui_sprite([tx - 2.0, ty], [rf.w, rf.h], rf.uv, WHITE);
                }
            }
            // `:42774-42776`: the cell itself, `White * num4`, width `- 2`.
            r.push_ui_sprite([tx, ty], [(cw - 2) as f32, f.h], f.uv, tint);
        }
    }
}

/// `Main.cs:42734-42758`: the hover text key for a team cell — `UI.TeamIcon`
/// plus the colour name, `_Active` when it is the player's own team else
/// `_Inactive`. Team 0 is `None`; 1..=5 are Red/Green/Blue/Yellow/Pink.
pub fn team_hover_key(i: i32, current_team: i32) -> String {
    let colour = match i {
        1 => "Red",
        2 => "Green",
        3 => "Blue",
        4 => "Yellow",
        5 => "Pink",
        _ => "None",
    };
    let suffix = if current_team == i {
        "_Active"
    } else {
        "_Inactive"
    };
    format!("UI.TeamIcon{colour}{suffix}")
}

/// `Main.cs:42685`: the PVP toggle's hover key from `hostile`.
pub fn pvp_hover_key(hostile: bool) -> &'static str {
    if hostile {
        "UI.PvpIcon_Active"
    } else {
        "UI.PvpIcon_Inactive"
    }
}

/// What a PVP/team cell press does — the effect half of
/// `Main.DrawPVPIcons`'s click arms (`Main.cs:42689-42694`, `:42753-42767`).
pub enum PvpClick {
    /// `hostile = !hostile` (`:42693`).
    ToggleHostile,
    /// `player.team = i` (`:42763`).
    SetTeam(i32),
}

/// The click gate chain, pure so the cooldown/team checks are testable without a
/// live net client:
///
/// * PVP toggle — `Main.mouseLeft && Main.mouseLeftRelease && Main.teamCooldown == 0`
///   (`:42689`); the state gates are already applied by [`pvp_hit`].
/// * team cell — the same mouse/cooldown pair plus
///   `player.team != i` (`:42753`) and `Player.TeamChangeAllowed()`, which is a
///   `return true` stub in 1.4.5.8 (`Player.cs:4517-4520`), so the
///   `Lang.misc[84]` refusal (`:42757-42759`) is unreachable and absent here.
pub fn pvp_click(state: &PvpIconState, hit: &PvpHit) -> Option<PvpClick> {
    if state.team_cooldown != 0 {
        return None;
    }
    match *hit {
        PvpHit::PvpToggle => Some(PvpClick::ToggleHostile),
        PvpHit::Team(i) => (state.team != i).then_some(PvpClick::SetTeam(i)),
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
pub fn draw_hotbar_lock(
    _r: &mut SpriteRenderer,
    _lookups: &assets::Lookups,
    push_side_tools_up: bool,
) {
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

    /// `Main.cs:42793/42805/42817`: a native 22x18 sheet at `(x, yPos)`, half-open.
    const SIZES: [(f32, f32); 3] = [(22.0, 18.0), (22.0, 18.0), (22.0, 18.0)];

    #[test]
    fn page_tab_hit_finds_each_tab_and_respects_the_half_open_edges() {
        let screen = Vec2::new(1600.0, 900.0);
        let y = 300.0;
        // (x, yPos) is the top-left; `+w` and `+h` are already outside.
        for (i, want) in [2, 1, 3].iter().enumerate() {
            let x = page_tab_x(screen.x, i);
            let inside = Vec2::new(x + 1.0, y + 1.0);
            assert_eq!(
                page_tab_hit(screen, y, inside, (0, 0), false, SIZES),
                Some(*want)
            );
            let corner = Vec2::new(x, y);
            assert_eq!(
                page_tab_hit(screen, y, corner, (0, 0), false, SIZES),
                Some(*want),
                "`Collision.CheckAABBvAABBCollision` is inclusive at the min corner"
            );
            let past = Vec2::new(x + 22.0, y + 1.0);
            assert_eq!(
                page_tab_hit(screen, y, past, (0, 0), false, SIZES),
                None,
                "... and exclusive at the max edge"
            );
            let below = Vec2::new(x + 1.0, y + 18.0);
            assert_eq!(page_tab_hit(screen, y, below, (0, 0), false, SIZES), None);
        }
    }

    #[test]
    fn page_tab_hit_gates_on_the_held_stack() {
        let screen = Vec2::new(1600.0, 900.0);
        let y = 300.0;
        let tab2 = Vec2::new(page_tab_x(screen.x, 0) + 4.0, y + 4.0);
        let tab1 = Vec2::new(page_tab_x(screen.x, 1) + 4.0, y + 4.0);
        // Anything held refuses every tab (`:42805`/`:42817`).
        assert_eq!(page_tab_hit(screen, y, tab2, (74, 3), false, SIZES), None);
        assert_eq!(page_tab_hit(screen, y, tab1, (74, 3), false, SIZES), None);
        // ... except tab 2 for a dye (`:42793` `Main.mouseItem.dye > 0`).
        assert_eq!(
            page_tab_hit(screen, y, tab2, (1007, 3), true, SIZES),
            Some(2)
        );
        assert_eq!(page_tab_hit(screen, y, tab1, (1007, 3), true, SIZES), None);
    }

    #[test]
    fn page_tab_click_toggles_back_to_page_zero() {
        // `:42846-42858`: a different tab selects it, the same tab clears to 0.
        assert_eq!(page_tab_click(2, 0), Some(2));
        assert_eq!(page_tab_click(2, 2), Some(0));
        assert_eq!(page_tab_click(1, 0), Some(1));
        assert_eq!(page_tab_click(1, 1), Some(0));
        assert_eq!(page_tab_click(2, 1), Some(2));
        // Tab 3 is the capture-mode entry, not a page switch (`:42835-42847`).
        assert_eq!(page_tab_click(3, 0), None);
        assert_eq!(page_tab_click(3, 3), None);
    }

    #[test]
    fn select_equip_page_follows_the_held_item() {
        // Air has no opinion (`ItemSlot.cs:4625`), so `Main.cs:42864-42866`
        // falls back to `EquipPageSelected`.
        assert_eq!(select_equip_page(0, 0, 2), -1);
        assert_eq!(select_equip_page(1, 0, 1), -1);
        // Armour and accessories force the equipment columns (`:4647-4650`).
        // 1 = Iron Pickaxe (not equipment), 4 = Iron Broadsword-equivalent
        // picks are data-driven below rather than hard-coded.
        let armour = (0u16..6196)
            .find(|&k| crate::itemdata::ITEMS[k as usize].head_slot != -1)
            .expect("a helmet exists");
        let acc = (0u16..6196)
            .find(|&k| crate::itemdata::ITEMS[k as usize].accessory)
            .expect("an accessory exists");
        assert_eq!(select_equip_page(armour, 1, 2), 0);
        assert_eq!(select_equip_page(acc, 1, 2), 0);
        assert_eq!(select_equip_page(1, 1, 2), -1, "a pickaxe forces nothing");
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

    /// A state with both gates open and no cooldown — the multiplayer client's
    /// shape (`Main.cs:2909-2919`).
    fn live_state() -> PvpIconState {
        PvpIconState {
            pvp_draw: true,
            team_select_draw: true,
            hide: false,
            hostile: false,
            team: 0,
            team_cooldown: 0,
        }
    }

    #[test]
    fn pvp_toggle_hover_box_is_strict() {
        // At 1600x900 the icon sits at (1373, 179), so `num2 = 1383`, `num3 = 179`
        // and the box is `1383-7 < x < 1383+25`, `179-2 < y < 179+37`
        // (`Main.cs:42678`).
        assert!(pvp_hover(1400.0, 200.0, 1383, 179));
        // The centre of the sprite is `num2 - 10 + 10 = 1383` — inside.
        assert!(pvp_hover(1383.0, 179.0, 1383, 179));
        // Every edge is strict: the four boundary values miss.
        assert!(!pvp_hover(1376.0, 200.0, 1383, 179));
        assert!(!pvp_hover(1408.0, 200.0, 1383, 179));
        assert!(!pvp_hover(1400.0, 177.0, 1383, 179));
        assert!(!pvp_hover(1400.0, 216.0, 1383, 179));
    }

    #[test]
    fn pvp_hit_prefers_the_toggle_then_the_cells() {
        let screen = Vec2::new(1600.0, 900.0);
        let st = live_state();
        // The toggle's own box.
        assert!(matches!(
            pvp_hit(
                screen,
                0,
                0.0,
                &st,
                Vec2::new(1390.0, 190.0),
                Some((20, 20))
            ),
            Some(PvpHit::PvpToggle)
        ));
        // Team cell 0 is at (1373, 239) — its own centre hits `Team(0)`.
        assert!(matches!(
            pvp_hit(
                screen,
                0,
                0.0,
                &st,
                Vec2::new(1380.0, 246.0),
                Some((20, 20))
            ),
            Some(PvpHit::Team(0))
        ));
        // Cell 5 is at (1393, 279) (`i/2 == 2` row).
        assert!(matches!(
            pvp_hit(
                screen,
                0,
                0.0,
                &st,
                Vec2::new(1400.0, 286.0),
                Some((20, 20))
            ),
            Some(PvpHit::Team(5))
        ));
        // The gap between the columns (x in [1393, 1393) is the next cell) and
        // well below the grid miss everything.
        assert!(pvp_hit(
            screen,
            0,
            0.0,
            &st,
            Vec2::new(1300.0, 500.0),
            Some((20, 20))
        )
        .is_none());
    }

    #[test]
    fn pvp_hit_respects_the_draw_gates() {
        let screen = Vec2::new(1600.0, 900.0);
        let over_toggle = Vec2::new(1390.0, 190.0);
        let over_team = Vec2::new(1380.0, 246.0);
        // `Main.ShouldPVPDraw` off (`Main.cs:2909`: `netMode != 1`) hides both.
        let mut st = live_state();
        st.pvp_draw = false;
        st.team_select_draw = false;
        assert!(pvp_hit(screen, 0, 0.0, &st, over_toggle, Some((20, 20))).is_none());
        assert!(pvp_hit(screen, 0, 0.0, &st, over_team, Some((20, 20))).is_none());
        // `Main.hidePVPAndTeamIcons` (`Main.cs:42660`) returns before the tests.
        let mut st = live_state();
        st.hide = true;
        assert!(pvp_hit(screen, 0, 0.0, &st, over_toggle, Some((20, 20))).is_none());
        // The toggle still works without the team grid (`ShouldTeamSelectDraw`
        // off, the single-player arm of `Main.cs:2911-2919`).
        let mut st = live_state();
        st.team_select_draw = false;
        assert!(matches!(
            pvp_hit(screen, 0, 0.0, &st, over_toggle, Some((20, 20))),
            Some(PvpHit::PvpToggle)
        ));
        assert!(pvp_hit(screen, 0, 0.0, &st, over_team, Some((20, 20))).is_none());
    }

    #[test]
    fn pvp_hit_follows_the_misc_equip_page_shift() {
        // `Main.cs:42670`: `EquipPage == 2` moves the whole group right by 46.
        let screen = Vec2::new(1600.0, 900.0);
        let st = live_state();
        let old = Vec2::new(1390.0, 190.0);
        assert!(pvp_hit(screen, 0, 0.0, &st, old, Some((20, 20))).is_some());
        assert!(pvp_hit(screen, 2, 0.0, &st, old, Some((20, 20))).is_none());
        let shifted = Vec2::new(old.x + 46.0, old.y);
        assert!(matches!(
            pvp_hit(screen, 2, 0.0, &st, shifted, Some((20, 20))),
            Some(PvpHit::PvpToggle)
        ));
    }

    #[test]
    fn team_hover_uses_the_narrowed_cell() {
        // `:42770-42772` gives each cell `rectangle2.Width - 2`; `Rectangle.Contains`
        // is half-open, so the last excluded column is `x + w - 2`.
        assert!(team_hover(Vec2::new(1373.0, 239.0), 1373, 239, 0, 20, 20));
        assert!(team_hover(Vec2::new(1390.0, 258.0), 1373, 239, 0, 20, 20));
        assert!(!team_hover(Vec2::new(1391.0, 250.0), 1373, 239, 0, 20, 20));
        assert!(!team_hover(Vec2::new(1380.0, 259.0), 1373, 239, 0, 20, 20));
        // Cell 1 starts one step of 20 px to the right.
        assert!(team_hover(Vec2::new(1393.0, 239.0), 1373, 239, 1, 20, 20));
    }

    #[test]
    fn pvp_hover_keys_follow_lang_keys() {
        // `Main.cs:42685`: the toggle says what the click will do.
        assert_eq!(pvp_hover_key(false), "UI.PvpIcon_Inactive");
        assert_eq!(pvp_hover_key(true), "UI.PvpIcon_Active");
        // `Main.cs:42734-42758`: `UI.TeamIcon{None,Red,Green,Blue,Yellow,Pink}`
        // with `_Active` on the player's own team, `_Inactive` otherwise.
        assert_eq!(team_hover_key(0, 0), "UI.TeamIconNone_Active");
        assert_eq!(team_hover_key(0, 3), "UI.TeamIconNone_Inactive");
        assert_eq!(team_hover_key(1, 1), "UI.TeamIconRed_Active");
        assert_eq!(team_hover_key(2, 1), "UI.TeamIconGreen_Inactive");
        assert_eq!(team_hover_key(3, 3), "UI.TeamIconBlue_Active");
        assert_eq!(team_hover_key(4, 0), "UI.TeamIconYellow_Inactive");
        assert_eq!(team_hover_key(5, 5), "UI.TeamIconPink_Active");
    }

    #[test]
    fn pvp_click_gates_on_the_cooldown_and_the_current_team() {
        let st = live_state();
        // `:42689`: the toggle flips whenever the cooldown is clear.
        assert!(matches!(
            pvp_click(&st, &PvpHit::PvpToggle),
            Some(PvpClick::ToggleHostile)
        ));
        // `:42753`: a different team is accepted…
        assert!(matches!(
            pvp_click(&st, &PvpHit::Team(3)),
            Some(PvpClick::SetTeam(3))
        ));
        // …but the player's own team is refused (`player.team != i`).
        assert!(pvp_click(&st, &PvpHit::Team(0)).is_none());
        let mut own = live_state();
        own.team = 3;
        assert!(pvp_click(&own, &PvpHit::Team(3)).is_none());
        assert!(matches!(
            pvp_click(&own, &PvpHit::Team(0)),
            Some(PvpClick::SetTeam(0))
        ));
        // A live `teamCooldown` (`:42672`, 0.25 tint) refuses everything.
        let mut cooling = live_state();
        cooling.team_cooldown = TEAM_COOLDOWN_LEN;
        assert!(pvp_click(&cooling, &PvpHit::PvpToggle).is_none());
        assert!(pvp_click(&cooling, &PvpHit::Team(4)).is_none());
        // One tick left is still closed — the C# compares against 0.
        cooling.team_cooldown = 1;
        assert!(pvp_click(&cooling, &PvpHit::PvpToggle).is_none());
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
        assert_eq!(map_icon_rect(0, Vec2::new(940.0, 900.0)), [500.0, 40.0]);
        assert_eq!(map_icon_rect(0, Vec2::new(939.0, 900.0)), [899.0, -200.0]);
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
