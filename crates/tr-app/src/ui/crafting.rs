//! The modern crafting panel: a faithful port of `NewCraftingUI`
//! (`decompiled/Terraria/GameContent/UI/NewCraftingUI.cs`) drawn as static chrome.
//!
//! P3 scope (spec §9.2): the cyan 9-slice frame, the title, the search bar, the
//! 11-tab filter strip, the separator, the recipe grid + auto-hidden
//! scrollbar, the detail area and the two toggles with their selected-crafting
//! indicator. The grid's entries are live (i46 craft): each cell runs the
//! `ItemGrid.DrawSlot` machine through [`super::crafting_grid`] (W4-f23
//! wiring) — the hover gate, the `Available` tint and the selected-recipe
//! `Inventory_Back24` overlay (`NewCraftingUI.cs:55-75`, `ItemSlot.cs:2371-2374`).
//!
//! Geometry: children are laid out against their parent's *inner* dimensions
//! (`UIElement.cs:287` `Recalculate`), i.e. the outer rect offset by the padding.

use glam::Vec2;
use tr_render::{SheetRect, SpriteRenderer};

use super::color;
use super::in_rect;
use super::panel;
use super::slot;
use super::text::{draw_border_string, hud_text_width, BorderFont};
use crate::assets;
use crate::item::{EMPTY_SLOT, ItemSlot};

/// Every static rectangle of the panel, derived from `screen` exactly as the
/// `UIElement` layout pass does (`UIElement.cs:306-322`). Kept pure for tests.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// `NewCraftingUI` itself: `20, 312, 490, screenH-350`
    /// (`NewCraftingUI.cs:300-306`, `Height = (-350, 1)` fill).
    pub state: [f32; 4],
    /// The background `UIPanel`'s outer rect (`:319-325`).
    pub panel: [f32; 4],
    /// The panel's content rect — padding 12 (`UIPanel.cs:32`), right 8
    /// (`NewCraftingUI.cs:324`).
    pub inner: [f32; 4],
    /// `UIText` draw origin, already shifted by `Y -= 2*textScale`
    /// (`:331-337`, `UIText.cs:138`).
    pub title: [f32; 2],
    /// `UIWrappedSearchBar` outer rect (`:338-342`, `UIWrappedSearchBar.cs:81`).
    pub search: [f32; 4],
    /// `UIImageButton` (`Button_Search_2`, 22x22, `UIImageButton.cs:31-32`).
    pub search_button: [f32; 4],
    /// The inner `UIPanel` — the text field (`UIWrappedSearchBar.cs:110-116`).
    pub search_box: [f32; 4],
    /// `SearchCancel` (24x24), a child of the *box* (`:142-147`).
    pub search_cancel: [f32; 4],
    /// `UICreativeItemsInfiniteFilteringOptions`
    /// (`UICreativeItemsInfiniteFilteringOptions.cs:40-44`).
    pub filter_strip: [f32; 4],
    /// `_gridContainer` (`:352-357` + `:407-415`).
    pub grid: [f32; 4],
    /// The `UIHorizontalSeparator` (`:359-363`).
    pub separator: [f32; 4],
    /// The `UIList` (`:367-373`).
    pub list: [f32; 4],
    /// The `UIScrollbar` (`:375-383`).
    pub scrollbar: [f32; 4],
    /// The detail-area origin = state inner TL + (24, 73) (`:438`).
    pub detail: [f32; 2],
    /// The grid toggle centre, `(42, 285)` or `(82, 285)` (`:484-490`).
    pub grid_toggle: [f32; 2],
    /// The nearby-chests toggle centre, 40 px right of the grid toggle
    /// (`:491-494`).
    pub nearby_toggle: [f32; 2],
}

/// `title_width` is `UIText("Crafting")`'s measured `_textSize.X`
/// (`NewCraftingUI.cs:424`), i.e. `hud_text_width(r, lookups, "Crafting", 1.0)`
/// (`crates/tr-app/src/ui/text.rs:67`). It is the only input beyond `screen`,
/// because the search bar is right-aligned and inset by the title (`:424-425`).
pub fn layout(screen: Vec2, title_width: f32, difficulty_3_and_creative_unblocked: bool) -> Layout {
    // `NewCraftingUI` (`:300-306`): Left 20, Top 312, Width 490, Height (-350, 1)
    // fill, padding 0. The parent state covers the whole screen
    // (`Main.cs:41706-41709`).
    let state = [20.0, 312.0, 490.0, screen.y - 350.0];
    let state_h = state[3];
    // The state's single child, `totalContainer`, fills it with no padding
    // (`:307-314`).

    // `uIPanel` (`:319-325`): Width fill (490), Height (-38, 1), Top 38,
    // `PaddingRight = 8`.
    let panel = [20.0, 312.0 + 38.0, 490.0, state_h - 38.0];
    // Panel padding: `SetPadding(_cornerSize = 12)` (`UIPanel.cs:32`), then
    // `PaddingRight = 8` (`NewCraftingUI.cs:324`).
    let inner = [
        panel[0] + 12.0,
        panel[1] + 12.0,
        panel[2] - 12.0 - 8.0,
        panel[3] - 12.0 - 12.0,
    ];

    // `UIText _text` (`:331-335`): Left -1, Top -2 inside the panel's inner rect;
    // `UIText.DrawSelf` then draws at `position.Y -= 2f * _textScale`
    // (`UIText.cs:138`).
    let title = [inner[0] - 1.0, inner[1] - 2.0 - 2.0];

    // `UIWrappedSearchBar` (`:338-342`): H 24 (`UIWrappedSearchBar.cs:81`), Top -4,
    // HAlign 1; Width = (-titleWidth - 10, 1) (`NewCraftingUI.cs:424`).
    let search_w = inner[2] - title_width - 10.0;
    let search = [
        inner[0] + inner[2] - search_w,
        inner[1] - 4.0,
        search_w,
        24.0,
    ];
    // The search button (`UIWrappedSearchBar.cs:102-105`): the 22x22
    // `Button_Search_2` (`UIImageButton.cs:31-32`), HAlign 0, VAlign 0.5.
    let search_button = [
        search[0],
        search[1] + (24.0 - 22.0) * 0.5,
        22.0,
        22.0,
    ];
    // The box panel (`:110-116`): W = (-22 - 3, 1), VAlign 0.5, HAlign 1.
    let box_w = search_w - 22.0 - 3.0;
    let search_box = [
        search[0] + search[2] - box_w,
        search[1],
        box_w,
        24.0,
    ];
    // The cancel button (`:142-147`): a child of the *box* (padding 0, `:124`),
    // 24x24 sheet, HAlign 1, VAlign 0.5, Left -2 — hence the fixed x.
    let search_cancel = [
        search_box[0] + search_box[2] - 24.0 - 2.0,
        search_box[1],
        24.0,
        24.0,
    ];

    // `_gridContainer` (`:352-357`): fill both, VAlign 1; Height = (-77, 1) with
    // `num = 77` (`:407-415`, the non-guide `UpdateCraftAreaSize`).
    let grid_h = inner[3] - 77.0;
    let grid = [
        inner[0],
        inner[1] + inner[3] - grid_h,
        inner[2],
        grid_h,
    ];

    // `UIHorizontalSeparator` (`:359-363`): W = (-8, 1), HAlign 0.5; H is the
    // sheet's height, 4 (`UIHorizontalSeparator.cs:28`).
    let sep_w = grid[2] - 8.0;
    let separator = [
        grid[0] + grid[2] * 0.5 - sep_w * 0.5,
        grid[1],
        sep_w,
        4.0,
    ];

    // The `UIList` (`:367-373`): W = (-20, 1), H = (-7, 1), VAlign 1, HAlign 0.
    let list_h = grid[3] - 7.0;
    let list = [
        grid[0],
        grid[1] + grid[3] - list_h,
        grid[2] - 20.0,
        list_h,
    ];

    // The `UIScrollbar` (`:375-383`): W 20 (`UIScrollbar.cs:59`), H = (-4*2 - 11, 1)
    // (`:379` with `num = 4f`, `:375`), Top -4, VAlign 1, HAlign 1.
    let sb_h = grid[3] - 19.0;
    let scrollbar = [
        grid[0] + grid[2] - 20.0,
        grid[1] + grid[3] - sb_h - 4.0,
        20.0,
        sb_h,
    ];

    // The filter strip (`:387-390`): HAlign 0.5 on `totalContainer`; H 40,
    // W 40 * 11 filters, Top 4
    // (`UICreativeItemsInfiniteFilteringOptions.cs:40-44`).
    let strip_w = 40.0 * 11.0;
    let filter_strip = [
        20.0 + 490.0 * 0.5 - strip_w * 0.5,
        312.0 + 4.0,
        strip_w,
        40.0,
    ];

    // The detail-area origin (`:438`): the state's inner TL + (24, 73); the state
    // has no padding, so inner == outer.
    let detail = [state[0] + 24.0, state[1] + 73.0];

    // The toggles (`:484-495`): `num2 = 42`, `num3 = 285`; `num2 += 40` when
    // `Main.LocalPlayer.difficulty == 3 && !Main.CreativeMenu.Blocked` (`:486-489`),
    // and the nearby toggle follows another 40 px right (`:491`).
    let toggle_x = if difficulty_3_and_creative_unblocked {
        42.0 + 40.0
    } else {
        42.0
    };
    let grid_toggle = [toggle_x, 285.0];
    let nearby_toggle = [toggle_x + 40.0, 285.0];

    Layout {
        state,
        panel,
        inner,
        title,
        search,
        search_button,
        search_box,
        search_cancel,
        filter_strip,
        grid,
        separator,
        list,
        scrollbar,
        detail,
        grid_toggle,
        nearby_toggle,
    }
}

/// `Main.OurFavoriteColor` (`Main.cs:66748`).
const FAVORITE_COLOR: [u8; 4] = [255, 231, 69, 255];
/// The indicator's `num = 16` offset (`Utils.cs:2737`).
const INDICATOR_BASE: f32 = 16.0;

/// `Utils.DrawSelectedCraftingBarIndicator` (`Utils.cs:2735-2745`): sixteen `32 x 2`
/// fade bars stacked upward from `craftY + 16`, then the solid `32 x 4` bar, in
/// draw order — `k = 16..=1` (later, brighter bars overpaint earlier ones), then
/// the solid bar. `FAVORITE_COLOR * m` goes through [`color::mul`] for the XNA
/// `Color * float` truncation.
///
/// The port has no `BlackTile`; push each piece on [`SpriteRenderer::white_uv`].
pub fn indicator_bars(craft_x: f32, craft_y: f32) -> [([f32; 4], [f32; 4]); 17] {
    let mut bars = [([0.0f32; 4], [0.0f32; 4]); 17];
    // `for (float num3 = num2; num3 > 0f; num3--)` (`Utils.cs:2740`).
    for i in 0..16usize {
        let num3 = (16 - i) as f32;
        // `new Rectangle(craftX - 16, craftY + num + (int)num3 * -1, 32, 2)`
        // (`:2741`), with `num = 16` (`:2737`).
        let rect = [craft_x - 16.0, craft_y + INDICATOR_BASE - num3, 32.0, 2.0];
        // `ourFavoriteColor * ((1f - num3 / num2) * 0.6f)` (`:2742`).
        let m = (1.0 - num3 / 16.0) * 0.6;
        bars[i] = (rect, color::tint(color::mul(FAVORITE_COLOR, m)));
    }
    // The solid `32 x 4` bar at `craftY + num` in `OurFavoriteColor` (`:2743-2744`).
    bars[16] = (
        [craft_x - 16.0, craft_y + INDICATOR_BASE, 32.0, 4.0],
        color::tint(FAVORITE_COLOR),
    );
    bars
}

/// The toggle hover test: `mouseX > craftX - 15 && mouseX < craftX + 15 &&
/// mouseY > craftY - 15 && mouseY < craftY + 15`
/// (`CraftingUI.cs:274`, `:339`) — strict on both sides.
fn toggle_hover(mouse_pos: Vec2, x: f32, y: f32) -> bool {
    (mouse_pos.x - x).abs() < 15.0 && (mouse_pos.y - y).abs() < 15.0
}

/// The click test for the `DrawCraftFromNearbyChestsToggle` button
/// (`CraftingUI.cs:336-355`): the same ±15 px square on the layout's
/// `nearby_toggle` position.
pub(crate) fn nearby_toggle_hit(screen: Vec2, mouse_pos: Vec2) -> bool {
    let l = layout(screen, 0.0, false);
    toggle_hover(mouse_pos, l.nearby_toggle[0], l.nearby_toggle[1])
}

/// The index of the filter tab under `mouse_pos`, if any — each tab is its
/// `filter_button_rect` square (`UICreativeItemsInfiniteFilteringOptions.cs:56-57`:
/// the 40 px pitch with the 46 px-tall frame hung off the strip's top).
pub(crate) fn filter_hit(screen: Vec2, mouse_pos: Vec2) -> Option<usize> {
    let l = layout(screen, 0.0, false);
    (0..11).find(|&i| in_rect(mouse_pos, filter_button_rect(l.filter_strip, i)))
}

/// The empty recipe slot: `Main.guideItem` / `recipe.createItem` carry no data in
/// P3, so every slot draws the empty `Inventory_Back18` background.
fn empty_slot() -> ItemSlot {
    EMPTY_SLOT
}

/// `ChatManager.DrawColorCodedStringWithShadow` (`ChatManager.cs:333-353`): four
/// black copies offset by `spread` along ±x/±y, then the coloured text on top —
/// `ui/text.rs`'s shared pass loop over the ui font.
fn hud_text_shadowed(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    spread: f32,
    text: &str,
) {
    draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        x,
        y,
        scale,
        &[[-spread, 0.0], [spread, 0.0], [0.0, -spread], [0.0, spread]],
        color,
        text,
    );
}

/// One centred toggle icon: `spriteBatch.Draw(tex, new Vector2(x, y), null,
/// Color.White, 0f, tex.Size() / 2f, 1f, ...)` (`CraftingUI.cs:290`, `:341`).
fn draw_centered_toggle(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<SheetRect>,
    x: f32,
    y: f32,
) {
    let Some(sheet) = sheet else {
        return;
    };
    let Some(uv) = lookups.hud_uv(Some(&sheet)) else {
        return;
    };
    let w = sheet.width as f32;
    let h = sheet.height as f32;
    r.push_ui_sprite([x - w * 0.5, y - h * 0.5], [w, h], uv, [1.0; 4]);
}

/// The measured `UIText("Crafting")` width without a renderer — the click
/// path's hit-test must agree with the draw pass's layout. The no-font
/// fallback estimates the 8-px bitmap grid (hit-test tolerance only).
fn title_width(lookups: &assets::Lookups) -> f32 {
    match &lookups.ui_font {
        Some(f) => super::text::font_width(f, "Crafting", 1.0),
        None => 8.0 * "Crafting".len() as f32,
    }
}

/// [`title_width`] for the App-side layout builders (the scroll frame in
/// `frame/draw.rs`) — the same measurement the draw and click paths use.
pub(crate) fn title_width_for(lookups: &assets::Lookups) -> f32 {
    title_width(lookups)
}

/// `UIDynamicItemCollection.UpdateSize` (`UIDynamicItemCollection.cs:160-165`):
/// `_itemsPerLine = Width / 44` (int division) and `MinHeight = 44 *
/// ceil(count / itemsPerLine)`. The grid element fills the `UIList`'s inner
/// list, so its width is [`Layout::list`]'s.
pub fn grid_cols(l: &Layout) -> usize {
    ((l.list[2] as i32) / 44).max(1) as usize
}

/// The `UIList`'s `_innerListHeight` for `n` shown recipes: the single grid
/// child's outer height (`UIList.RecalculateChildren` `UIList.cs:116-128` —
/// one item, no `ListPadding`), which is the fill height raised to the
/// grid's `MinHeight` (`UpdateSize` `:164`). Never below the list's own
/// height, so a short grid never scrolls.
pub fn grid_content_height(l: &Layout, n: usize) -> f32 {
    let cols = grid_cols(l);
    let rows = (n + cols - 1) / cols;
    l.list[3].max(44.0 * rows as f32)
}

/// `GetGridParameters`' start half (`UIDynamicItemCollection.cs:83-91`):
/// `(startX, startY, startItemIndex)` for the scrolled grid. `rect` is the
/// grid's rect — the inner list lifted by `-ViewPosition`
/// (`UIList.DrawSelf` `UIList.cs:181`); `viewCullingArea` is the list's own
/// rect (`UIInnerList.GetViewCullingArea` `UIList.cs:34-37`). Both truncate
/// to ints (`ToRectangle`), and `num = (min(cullTop, rectTop) - cullTop) / 44`
/// is C# int division (toward zero). The start row snaps in 44 px steps
/// while `startY` rides the truncated lifted top — the fractional scroll
/// remainder shifts every drawn cell.
fn grid_start(l: &Layout, scroll: f32) -> (f32, i32, i32) {
    let (rx, rw) = (l.list[0] as i32, l.list[2] as i32);
    let cols = grid_cols(l) as i32;
    // `startX = rectangle.Center.X - (int)(44 * _itemsPerLine * 0.5f)` (`:85`).
    let start_x = rx as f32 + rw as f32 * 0.5 - (44 * cols) as f32 * 0.5;
    let rect_top = (l.list[1] - scroll) as i32;
    let cull_top = l.list[1] as i32;
    let num = (cull_top.min(rect_top) - cull_top) / 44;
    (start_x, rect_top - num * 44, -num * cols)
}

/// `GetGridParameters`' culling window (`UIDynamicItemCollection.cs:87-96`):
/// `[startItemIndex, endItemIndex)` — the rows `DrawSelf` walks (and so the
/// only cells whose `DrawSlot` hover/click fires). `endItemIndex =
/// min(count, ceil(cullHeight / 44) * cols + startItemIndex + cols)` (`:92-96`).
/// The quirk is faithful: a top row with only a sub-44px sliver visible is
/// culled until the scroll crosses the next 44 px step.
pub fn grid_cull_range(l: &Layout, n: usize, scroll: f32) -> (usize, usize) {
    let cols = grid_cols(l) as i32;
    let (_, _, start) = grid_start(l, scroll);
    let cull_h = l.list[3] as i32;
    // `(int)Math.Ceiling(cullHeight / 44f) * cols` (`:92`).
    let num2 = ((cull_h as f32 / 44.0).ceil() as i32) * cols;
    let end = (n as i32).min(num2 + start + cols);
    ((start.max(0) as usize).min(n), (end.max(0) as usize).min(n))
}

/// The `i`-th recipe cell's origin and 44x44 hover/click rect —
/// `UIDynamicItemCollection.GetGridParameters` (`:81-97`) +
/// `GetItemSlotHitbox` (`:75-79`): the grid element fills the `UIList`'s
/// inner list (`UIList.cs:57` Width/Height fill, `:122-123` single item at
/// Top 0) lifted by `-ViewPosition` (`UIList.cs:181`); `startX = Center.X -
/// (int)(44 * _itemsPerLine * 0.5f)` (the rows are centred, int-truncated),
/// each cell is `startX + num % cols * 44, startY + num / cols * 44` —
/// **integer** division (toward zero, so rows above the cull start keep
/// their hitboxes), 44x44 (`:78`). `scroll` is the live
/// `UIScrollbar.ViewPosition`; the shared geometry for the draw pass and
/// `hit_test`.
pub fn recipe_cell(l: &Layout, i: usize, scroll: f32) -> [f32; 4] {
    let (start_x, start_y, start) = grid_start(l, scroll);
    let cols = grid_cols(l) as i32;
    let num = i as i32 - start;
    [
        start_x + (num % cols) as f32 * 44.0,
        (start_y + num / cols * 44) as f32,
        44.0,
        44.0,
    ]
}

/// `UIDynamicItemCollection.DrawSelf` (`UIDynamicItemCollection.cs:40`):
/// `Main.inventoryScale = 3465f / 4095f` — the grid's slot scale for the
/// whole walk. 3465/4095 = 11/13 exactly, so the 52 px `Inventory_Back*`
/// sheets draw at 52 * 11/13 = 44 px — the cell background fills the 44x44
/// hitbox with no stretch (`ItemSlot.cs:2355`).
pub const GRID_INVENTORY_SCALE: f32 = 3465.0 / 4095.0;

/// The recipe grid cell the mouse is over, mapped back to the available
/// recipe list (`NewCraftingUI`'s clickable recipe slots). Cells are 44x44 on
/// a 44 px pitch, centred on the list rect ([`recipe_cell`]) and lifted by
/// the live scroll; only the `GetGridParameters` culling window is walked
/// (`DrawSlot` — and so `HandleCraftSlot` — never fires outside it), and a
/// cell must sit inside the list rect: `base.IsMouseHovering`
/// (`UIDynamicItemCollection.cs:52`) walks up through the `UIList`'s
/// `ContainsPoint` (`UIElement.cs:354-357`), which the lifted grid does not
/// override (`UIInnerList`'s always-true `ContainsPoint` is the inner
/// list's, one level below the grid).
pub(crate) fn hit_test(
    screen: Vec2,
    lookups: &assets::Lookups,
    mouse: Vec2,
    available: &[usize],
    scroll: f32,
) -> Option<usize> {
    let l = layout(screen, title_width(lookups), false);
    let scroll = clamp_scroll(scroll, &l, available.len());
    let (start, end) = grid_cull_range(&l, available.len(), scroll);
    if !in_rect(mouse, l.list) {
        return None;
    }
    for i in start..end {
        // The base walk's hover gate (`UIDynamicItemCollection.cs:52` via
        // `crafting_grid::slot_hovering`): int-truncated mouse, exclusive
        // right/bottom, on the same 44x44 cell rect the draw hovers.
        if super::crafting_grid::slot_hovering(
            true,
            recipe_cell(&l, i, scroll).map(|v| v as i32),
            [mouse.x, mouse.y],
            false,
        ) {
            return Some(available[i]);
        }
    }
    None
}

/// `UIScrollbar.ViewPosition`'s clamp for the craft grid — the shared
/// `char_select::clamp_scroll` over the grid's content height
/// ([`grid_content_height`], `UIList.UpdateScrollbar` `:130-137` →
/// `SetView` `UIScrollbar.cs:72-78`).
pub(crate) fn clamp_scroll(scroll: f32, l: &Layout, n: usize) -> f32 {
    super::char_select::clamp_scroll(scroll, l.list[3], grid_content_height(l, n))
}

/// `UIList.ScrollWheel` (`UIList.cs:107-114`) for the craft grid:
/// `ViewPosition -= evt.ScrollWheelValue`, one notch = 120 raw units
/// (`char_select::WHEEL_STEP`), clamped by `SetView`.
pub(crate) fn scroll_wheel(scroll: f32, delta: i32, l: &Layout, n: usize) -> f32 {
    super::char_select::scroll_wheel(scroll, delta, l.list[3], grid_content_height(l, n))
}

/// Whether the wheel reaches the craft grid — `UserInterface.Update`
/// dispatches to `GetElementAt` (`:219-221`): the whole list rect (the
/// `UIInnerList`'s always-true `ContainsPoint`, `UIList.cs:16-19`) plus the
/// attached scrollbar (`UIScrollbar` is its own target). The bar attaches
/// only while the content overflows (`CanScroll`, `UIScrollbar.cs:49` via
/// `SetView`'s clamp).
pub(crate) fn wheel_target(l: &Layout, n: usize, mouse: Vec2) -> bool {
    let content_h = grid_content_height(l, n);
    in_rect(mouse, l.list)
        || (super::char_select::scrollbar_attaches(l.list[3], content_h)
            && in_rect(mouse, l.scrollbar))
}

/// `UIScrollbar.LeftMouseDown` (`UIScrollbar.cs:126-143`) for the craft
/// grid's bar: `(new ViewPosition, drag latch)` — `Some(offset)` grabs the
/// handle (`_dragYOffset = mouse.Y - handle.Y`, `:134-135`), `None` is the
/// track jump (`:137-141`). `None` overall = the press was not the bar's.
pub(crate) fn scrollbar_press(
    l: &Layout,
    n: usize,
    scroll: f32,
    mouse: Vec2,
) -> Option<(f32, Option<f32>)> {
    let content_h = grid_content_height(l, n);
    if !super::char_select::scrollbar_attaches(l.list[3], content_h)
        || !in_rect(mouse, l.scrollbar)
    {
        return None;
    }
    let handle = panel::scrollbar_handle_at(l.scrollbar, l.list[3], content_h, scroll);
    if in_rect(mouse, handle) {
        return Some((scroll, Some(mouse.y - handle[1])));
    }
    Some((
        panel::scrollbar_jump_value(l.scrollbar, l.list[3], content_h, handle[3], mouse.y),
        None,
    ))
}

/// The drag frame (`UIScrollbar.DrawSelf`, `UIScrollbar.cs:109-112`): while
/// `_isDragging` the position follows the pointer with the latched offset.
pub(crate) fn scrollbar_drag(l: &Layout, n: usize, drag_offset: f32, mouse_y: f32) -> f32 {
    panel::scrollbar_drag_value(
        l.scrollbar,
        l.list[3],
        grid_content_height(l, n),
        drag_offset,
        mouse_y,
    )
}

/// Draws the whole panel in the source's z-order. `mouse_pos` only feeds the hover
/// frames; `guide_mode` selects the `Main.InGuideCraftMenu` variants;
/// `filter_flags` is `EntryFilterer.ActiveFilters` as the strip's eleven slots
/// (`UICreativeItemsInfiniteFilteringOptions.cs:104-110`).
/// `craft_raw` is `Main.availableRecipe[..numAvailableRecipes]` (the RAW list,
/// `Recipe.UpdateRecipeList`), `craft_shown` its `_filteredRecipes` narrowing
/// (`NewCraftingUI.UpdateContents` `:596-611`); with no filter the two are
/// equal. `selected_recipe_index` is `NewCraftingUI._selectedRecipeIndex`
/// (`:196`, `int?` — the port derives it from `Main.focusRecipe` via
/// `availableRecipe[focusRecipe]`, `OnActivate` `:904`) and `inventory_back`
/// is `Main.inventoryBack`, the selected-cell overlay's tint
/// (`ItemSlot.cs:2373`). `scroll` is the live `UIScrollbar.ViewPosition`
/// (`UIList.cs:181` lifts the grid by it) and `scroll_hover` is the
/// `_isDragging || _isHoveringOverHandle` latch (`UIScrollbar.cs:122`).
#[allow(clippy::too_many_arguments)]
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    mouse_pos: Vec2,
    craft_raw: &[usize],
    craft_shown: &[usize],
    guide_mode: bool,
    difficulty_3_and_creative_unblocked: bool,
    filter_flags: [bool; 11],
    selected_recipe_index: Option<usize>,
    inventory_back: [f32; 4],
    scroll: f32,
    scroll_hover: bool,
    craft_from_nearby_chests: bool,
) {
    // `UpdateText` (`:417-427`): the search bar's width is inset by the title's
    // measured outer width, so measure it before building the layout.
    let title_width = hud_text_width(r, lookups, "Crafting", 1.0);
    let l = layout(screen, title_width, difficulty_3_and_creative_unblocked);

    // 1. The panel frame (`UIPanel.cs:68-77`): background, then border.
    //    bg = `ShiftBlueToCyanTheme((63,82,151) * 0.7f) * 0.8f`
    //    (`UIPanel.cs:20` default, `NewCraftingUI.cs:326,328`).
    let bg = color::tint(color::mul(
        color::shift_blue_to_cyan(color::mul([63, 82, 151, 255], 0.7)),
        0.8,
    ));
    //    border = `ShiftBlueToCyanTheme(Color.Black) * 0.8f`
    //    (`UIPanel.cs:24` default, `NewCraftingUI.cs:327,329`).
    let border = color::tint(color::mul(color::shift_blue_to_cyan([0, 0, 0, 255]), 0.8));
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, bg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.panel, border);

    // 2. The title, `_text` = `Lang.inter[25]` "Crafting" (`:419`), drawn before the
    //    search bar (`:336` vs `:350`), white with a 4-direction black shadow at
    //    spread 1.5 (`UIText.cs:158`).
    hud_text_shadowed(
        r,
        lookups,
        l.title[0],
        l.title[1],
        1.0,
        [1.0; 4],
        1.5,
        "Crafting",
    );

    // 3. The search bar (`:338-342`): the magnifier button, the box panel, the
    //    `Name:` placeholder and the cancel button (`UIWrappedSearchBar.cs`).
    draw_search_bar(r, lookups, &l, mouse_pos);

    // 4. The separator (`:359-365`): colour =
    //    `ShiftBlueToCyanTheme(new Color(89, 116, 213, 255) * 0.9f)`.
    panel::draw_horizontal_separator(
        r,
        lookups,
        l.separator[0],
        l.separator[1],
        l.separator[2],
        color::tint(color::shift_blue_to_cyan(color::mul([89, 116, 213, 255], 0.9))),
    );

    // 5. The grid area (`UIDynamicItemCollection.cs:38-71`): the shown recipes
    //    (`_filteredRecipes`) as 44x44 cells centred on the list rect
    //    (`GetGridParameters` `:85-86` + `GetItemSlotHitbox` `:75-79` — integer
    //    `num / _itemsPerLine`) — the same cells `hit_test` maps clicks back to
    //    ([`recipe_cell`]). Each cell runs the nested `ItemGrid.DrawSlot`
    //    machine (`NewCraftingUI.cs:55-75`, W4-f23 wiring): `RecipeEntry`
    //    carries the raw index + the RAW `availableRecipe` position
    //    (`UpdateContents` `:610` — the shown position only with no filter),
    //    the base walk's hover gate decides the hover, and the machine picks
    //    the selection overlay + the White-vs-DisabledSlotColor tint. The port
    //    is mouse-only (`using_gamepad = false`, `current_point = 0` — gamepad
    //    unported, research/18 dev. 10). The cells ride the live scroll: the
    //    grid rect is the inner list lifted by `-ViewPosition`
    //    (`UIList.cs:181`), `GetGridParameters` culls to the visible window
    //    (`:87-96`) and the `UIList`'s `OverflowHidden` scissor
    //    (`UIList.cs:59`, `UIElement.cs:222-237`) clips the straddling rows.
    let scroll = clamp_scroll(scroll, &l, craft_shown.len());
    let (start, end) = grid_cull_range(&l, craft_shown.len(), scroll);
    r.push_clip(l.list);
    for i in start..end {
        let recipe_idx = craft_shown[i];
        let cell = recipe_cell(&l, i, scroll);
        let mut entry = super::crafting_grid::RecipeEntry::new(recipe_idx);
        // `UpdateContents` `:610`: `availableIndex = i` walks the RAW
        // `Main.availableRecipe` list — with a filter active the shown
        // position diverges from it (W4-f23 deviation fix).
        entry.available_index = super::crafting_grid::available_position(craft_raw, recipe_idx);
        entry.grid_index = i as i32; // `:606` (the shown list is `_filteredRecipes`)
        // `base.IsMouseHovering` needs the pointer inside the LIST too
        // (`UIElement.ContainsPoint` up the parent chain) — a cell lifted
        // above the list's top is drawn clipped but not hoverable.
        let hovering = in_rect(mouse_pos, l.list)
            && super::crafting_grid::slot_hovering(
                true,
                cell.map(|v| v as i32),
                [mouse_pos.x, mouse_pos.y],
                false,
            );
        let d = super::crafting_grid::draw_slot(&entry, selected_recipe_index, false, 0, hovering);
        // `ItemSlot.Draw`'s context-41 background (`ItemSlot.cs:2298-2302`,
        // `:2340-2343`, `:2355`): `Inventory_Back18` at the hitbox's TopLeft,
        // tint `ShiftBlueToCyanTheme((16,36,56,180)*0.9f)` × lightColor — the
        // DrawSlot lightColor is `entry.Available ? White : DisabledSlotColor`
        // (`NewCraftingUI.cs:74`), multiplied per-channel `(a*b)>>8` (XNA
        // `Color.MultiplyRGBA`). `Main.inventoryScale = 3465/4095`
        // (`UIDynamicItemCollection.cs:40`) makes the 52 px sheet exactly
        // 44 px — the cell fills its hitbox, no stretch.
        let back_tint = color::tint(crate::render::special_tiles::xna_multiply_rgba(
            color::shift_blue_to_cyan(color::mul([16, 36, 56, 180], 0.9)),
            d.tint,
        ));
        slot::draw_slot(
            r,
            lookups,
            &empty_slot(),
            cell[0],
            cell[1],
            41,
            0,
            false,
            GRID_INVENTORY_SCALE,
            back_tint,
        );
        // `DrawSelectionHighlightForGridSlot` (`:66`) → the context-41
        // `Inventory_Back24` overlay in `Main.inventoryBack`
        // (`ItemSlot.cs:2371-2374`), at the slot position × inventoryScale.
        if d.grid_selection_highlight {
            draw_grid_selection_overlay(r, lookups, cell, inventory_back);
        }
        // `HandleCraftSlot` (`:72`) fires on the hover edge; the port's hover
        // cue is the item tooltip (`draw_hud`'s `hit_test`), so the hover
        // itself draws nothing extra.
        let recipe = entry.recipe();
        // No `Item_*` sheets are extracted yet: placeable items fall back to
        // the tile's plain 16x16 frame, the same fallback the world drops use
        // (`item.rs`'s drop draw).
        let icon = lookups
            .item_icon(recipe.create.0)
            .or_else(|| {
                let t = crate::registry::get(recipe.create.0)?.tile?;
                Some((lookups.plain_tex(t)?.uv, 16.0, 16.0))
            });
        if let Some((uv, w, h)) = icon {
            // `ItemSlot.DrawItemIcon` (`ItemSlot.cs:2515`, `:2880-2881`) via
            // `DrawItem_GetColorAndScale` (`:2966-2975`): the icon centres on
            // `position + value.Size()/2 * inventoryScale` (`:2499`, `:2515`)
            // — the cell's centre — at `inventoryScale * fit`. The fit runs
            // only when width OR height exceeds the 32 px sizeLimit, and the
            // denominator is the height when `Width <= Height`, else the
            // width (`:2967-2969`). The tint is `DrawSlot`'s lightColor
            // (`NewCraftingUI.cs:74`), which context 41 multiplies into the
            // icon (`ItemSlot.cs:2340-2343`).
            let fit = if w > 32.0 || h > 32.0 {
                if w <= h { 32.0 / h } else { 32.0 / w }
            } else {
                1.0
            };
            let s = GRID_INVENTORY_SCALE * fit;
            let (dw, dh) = (w * s, h * s);
            r.push_ui_sprite(
                [cell[0] + cell[2] * 0.5 - dw * 0.5, cell[1] + cell[3] * 0.5 - dh * 0.5],
                [dw, dh],
                uv,
                super::crafting_grid::slot_tint(&entry),
            );
        } else {
            // Items without an extracted sprite (the atlas carries only the
            // world's used tile frames): a grey placeholder box — documented
            // in docs/ANALYSIS.md.
            r.push_ui_sprite([cell[0] + 6.0, cell[1] + 6.0], [32.0, 32.0], r.white_uv(), [0.5; 4]);
        }
    }
    r.pop_clip();

    // 6. The scrollbar (`:376-384`): `UIList.UpdateScrollbar`
    //    (`UIList.cs:130-137`) feeds `SetView(listHeight, innerListHeight)`
    //    — the grid's content height ([`grid_content_height`]) — and
    //    `AutoHide` hides the bar when it cannot scroll
    //    (`UIScrollbar.cs:105`, `CanScroll` `:49`). The handle rides the
    //    live `ViewPosition` and the hover/drag latch's 1-vs-0.85 alpha
    //    (`:122`).
    let content_h = grid_content_height(&l, craft_shown.len());
    panel::draw_scrollbar_handle_alpha(
        r,
        lookups,
        l.scrollbar,
        l.list[3],
        content_h,
        scroll,
        true,
        if scroll_hover { 1.0 } else { 0.85 },
    );

    // 7. The filter strip (`:387-392`), drawn over the panel subtree, matching
    //    `totalContainer`'s child order (`:330`, `:392`); `active` is the
    //    filterer's `ActiveFilters` as `UpdateVisuals` reads it (`:104`).
    draw_filter_strip(r, lookups, &l, mouse_pos, filter_flags);

    // 8. The detail area (`DrawChildren`, `:438-482`), drawn after the base pass so
    //    it sits on top of everything.
    draw_detail_area(r, lookups, &l, guide_mode);

    // 9. The toggles (`:484-495`).
    draw_toggles(r, lookups, &l, mouse_pos, guide_mode, craft_from_nearby_chests);
}

/// The selected-recipe cell's `Inventory_Back24` overlay
/// (`ItemSlot.cs:2371-2374`): `context == 41 &&
/// ItemSlot.DrawSelectionHighlightForGridSlot` draws the sheet at the slot
/// position in `Main.inventoryBack` at `Main.inventoryScale`. The grid's
/// scale is 3465/4095 = 11/13 (`UIDynamicItemCollection.cs:40`), so the 52 px
/// sheet draws at exactly the 44x44 cell — no stretch.
fn draw_grid_selection_overlay(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    cell: [f32; 4],
    inventory_back: [f32; 4],
) {
    let Some(sheet) = lookups.ui_back24 else {
        return;
    };
    super::push_whole(r, lookups, &sheet, cell, inventory_back);
}

/// `DrawChildren`'s detail area (`NewCraftingUI.cs:438-482`): the craft slot and
/// the `Images/UI/Craft` icon, plus the guide-mode slot and label.
///
/// Deviation (documented): vanilla draws the craft slot/icon only when a recipe is
/// selected (`:449` `if (recipeEntry != null)`); with `Recipe.maxRecipes == 0` the
/// port always draws them as static chrome.
fn draw_detail_area(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    guide_mode: bool,
) {
    let (dx, dy) = (l.detail[0], l.detail[1]);

    // Guide variants come first (`:439-446`).
    if guide_mode {
        // `DrawRecipeSlot(spriteBatch, Main.guideItem, 7, vector + (0,58), true, 1f)`
        // (`:441`). `DrawRecipeSlot` draws with `Main.inventoryBack = Color.White *
        // (191f / 255f)` (`:560`), which context 7 picks up (`ItemSlot.cs:2149`).
        let guide_tint = [191.0 / 255.0; 4];
        slot::draw_slot(
            r,
            lookups,
            &empty_slot(),
            dx,
            dy + 58.0,
            7,
            -1,
            false,
            1.0,
            guide_tint,
        );
        // The label at `vector + (52, 73)` (`:445`): `Main.guideItem.IsAir ?
        // Lang.inter[24] : ...` — the guide slot is always empty in the port, so
        // `Lang.inter[24]` = "Place a material here", spread 1.
        hud_text_shadowed(
            r,
            lookups,
            dx + 52.0,
            dy + 73.0,
            1.0,
            [1.0; 4],
            1.0,
            "Place a material here",
        );
    }

    // The craft slot (`:456`): `DrawRecipeSlot(..., recipe.createItem, 42, vector,
    // ..., 1f)` — the `Inventory_Back18` crafting tint with `lightColor = White`
    // (`:563`, `ItemSlot.cs:2340-2343`), i.e. `craft_slot_tint()` as-is.
    slot::draw_slot(
        r,
        lookups,
        &empty_slot(),
        dx,
        dy,
        42,
        -1,
        false,
        1.0,
        slot::craft_slot_tint(),
    );

    // `Images/UI/Craft` at `vector + (47, 13)`, scale 0.8, White (`:460`).
    if let Some(sheet) = lookups.ui_craft_icon.as_ref() {
        if let Some(uv) = lookups.hud_uv(Some(sheet)) {
            r.push_ui_sprite(
                [dx + 47.0, dy + 13.0],
                [sheet.width as f32 * 0.8, sheet.height as f32 * 0.8],
                uv,
                [1.0; 4],
            );
        }
    }
}

/// `DrawChildren`'s toggles (`NewCraftingUI.cs:484-495`): the grid toggle with its
/// selected-crafting indicator, then the nearby-chests toggle (hidden in the guide
/// menu, `:492-494`).
fn draw_toggles(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &Layout,
    mouse_pos: Vec2,
    guide_mode: bool,
    craft_from_nearby_chests: bool,
) {
    // `CraftingUI.DrawGridToggle` (`CraftingUI.cs:267-334`).
    let (gx, gy) = (l.grid_toggle[0], l.grid_toggle[1]);
    let hover = toggle_hover(mouse_pos, gx, gy);
    // The indicator is drawn first, when `Main.PipsCurrentPage == Recipes`
    // (`CraftingUI.cs:275-277`) — the crafting panel's page.
    for (rect, col) in indicator_bars(gx, gy) {
        r.push_ui_sprite([rect[0], rect[1]], [rect[2], rect[3]], r.white_uv(), col);
    }
    // Frame `num = 4` because `NewCraftingUI.Visible` (`CraftingUI.cs:281-283`),
    // `+1` when hovered (`:289`); drawn centred with origin `Size()/2` (`:290`).
    let num = 4 + hover as usize;
    draw_centered_toggle(r, lookups, lookups.craft_toggles[num], gx, gy);

    // `DrawCraftFromNearbyChestsToggle` (`CraftingUI.cs:336-355`), only outside the
    // guide menu (`NewCraftingUI.cs:492-494`).
    if !guide_mode {
        let (nx, ny) = (l.nearby_toggle[0], l.nearby_toggle[1]);
        let hover = toggle_hover(mouse_pos, nx, ny);
        // `num = 2 - CraftFromNearbyChests.ToInt() * 2 + hover`
        // (`CraftingUI.cs:340`): ON → 0/1 (the open chest), OFF → 2/3 (the
        // closed chest). The setting defaults to true (`Player.cs:406`) but
        // the toggle flips it, so the frame reads the LIVE setting — the old
        // `num = hover` stand-in drew the open chest even when the setting
        // was off (W4-f23 closeout item 4).
        let num = 2 - (craft_from_nearby_chests as usize) * 2 + hover as usize;
        draw_centered_toggle(r, lookups, lookups.chest_craft[num], nx, ny);
    }
}

/// `Main.DoStatefulTickSound(ref bool flag, bool state)` (`Main.cs:3237-3247`):
/// the latch follows the state, and the enter edge (`false → true`) plays
/// `SoundEngine.PlaySound(12)`. The nearby-chests toggle's hover cue
/// (`CraftingUI.cs:354`, `ref Main.nearbyCraftingMouseOver`) rides it.
/// Returns whether the cue plays so the caller owns the audio side effect.
pub fn stateful_tick_sound(flag: &mut bool, state: bool) -> bool {
    if *flag != state {
        *flag = state;
        if *flag {
            return true; // `:3244` — SoundEngine.PlaySound(12)
        }
    }
    false
}

// ---- Task 3: the filter strip and the search bar ----

/// The icon frame of each of the 11 filters, in `NewCraftingUI`'s order —
/// Weapon, Armor, Vanity, BuildingBlock, Furniture, Accessories,
/// MiscAccessories, Consumables, Tools, Materials, MiscFallback
/// (`NewCraftingUI.cs:282-298`) — from `ItemFilters.GetImage`:
/// Weapon `ItemFilters.cs:206` `Frame(11)`, Armor `:241` `Frame(11, 1, 2)`,
/// Vanity `:264` 8, BuildingBlock `:98` 4, Furniture `:126` 7, Accessories
/// `:319` 1, MiscAccessories `:342` 9, Consumables `:375` 3, Tools `:183` 6,
/// Materials `:458` 10, MiscFallback `:435` 5. Not sequential.
pub const FILTER_ICON_K: [usize; 11] = [0, 2, 8, 4, 7, 1, 9, 3, 6, 10, 5];

/// The `i`-th filter button (`:57` `Left.Set(num * i, 0f)` with `num = 40`,
/// `:40`): 40 px pitch, and the 40x46 tab frame (`:55-56`
/// `Frame(2, 4).OffsetSize(-2, -2)` on the 84x192 `Infinite_Tabs_B_2` sheet) hung
/// off the strip's top — it overflows the 40 px-tall strip by 6 px.
pub fn filter_button_rect(strip: [f32; 4], i: usize) -> [f32; 4] {
    [strip[0] + 40.0 * i as f32, strip[1], 40.0, 46.0]
}

/// The filter's icon inside button `i`: 28x30 (`ItemFilters.GetImage`
/// `Frame(11, 1, k).OffsetSize(-2, 0)` on the 330x30 `Infinite_Icons` sheet),
/// `Left 6` and `HAlign 0` (`UICreativeItemsInfiniteFilteringOptions.cs:65-66`),
/// `VAlign 0.5` from the `GetImage()` object initializers
/// (`ItemFilters.cs:101-102` and every other `GetImage`) — hence 8 px below the
/// button top: `(46 - 30) / 2`.
pub fn filter_icon_rect(strip: [f32; 4], i: usize) -> [f32; 4] {
    let b = filter_button_rect(strip, i);
    [b[0] + 6.0, b[1] + (46.0 - 30.0) * 0.5, 28.0, 30.0]
}

/// `UpdateVisuals` (`UICreativeItemsInfiniteFilteringOptions.cs:106`):
/// `SetFrame(2, 4, flag.ToInt(), flag.ToInt() * 2 + isMouseHovering.ToInt(), -2, -2)`
/// — the frame column is the active flag, the row is `active*2 + hover`.
pub fn filter_frame_col_row(active: bool, hover: bool) -> (usize, usize) {
    let flag = active as usize;
    (flag, flag * 2 + hover as usize)
}

/// The `Infinite_Tabs_B_2` source rect for frame `(col, row)`:
/// `SetFrame(2, 4, ...)` cells are 42x48 on the 84x192 sheet (`:28-30`
/// `barFramesX = 2`, `barFramesY = 4`), and `OffsetSize(-2, -2)` (`:106`,
/// `UIImageFramed.cs:40-43`) trims them to 40x46.
pub fn filter_button_src(col: usize, row: usize) -> [f32; 4] {
    [42.0 * col as f32, 48.0 * row as f32, 40.0, 46.0]
}

/// The `Infinite_Icons` source rect for icon `k`: `Frame(11, 1, k)` cells are
/// 30x30 on the 330x30 sheet, and `OffsetSize(-2, 0)` trims them to 28x30
/// (`ItemFilters.cs:98` and every other `GetImage`).
pub fn filter_icon_src(k: usize) -> [f32; 4] {
    [30.0 * k as f32, 0.0, 28.0, 30.0]
}

/// The `Name:` placeholder's draw position (`UITextPanel.TextDrawPosition`,
/// `UITextPanel.cs:92-109`). The inner `UITextBox` fills the 24 px box
/// (`UIWrappedSearchBar.cs:128-131`) but `SetText` pins
/// `MinHeight = 16*0.8 + 12 + 12 = 36.8` (`UITextPanel.cs:132-134`), which clamps
/// the 24 px height up to 36.8; `VAlign 0.5` (`:131`) centres that in the box, the
/// box's own padding puts the text 12 px in (`UIPanel.cs:32` — the box
/// `SetPadding(0)`s itself, `UIWrappedSearchBar.cs:124`, but the `UITextBox` keeps
/// the `UIPanel` default), and the draw shifts `Y -= 2*0.8` (`UITextPanel.cs:105`):
/// `y = box.y + (24 - 36.8)/2 + 12 - 1.6 = box.y + 4.0`, `x = box.x + 12`.
pub fn search_text_pos(l: &Layout) -> (f32, f32) {
    (l.search_box[0] + 12.0, l.search_box[1] + 4.0)
}

/// Pushes one sprite cut out of `sheet` by `src` (in the sheet's own pixels) at
/// the destination `dst` (`[x, y, w, h]`). Draws nothing for a missing sheet, so
/// `Lookups::default()` renders a blank frame (the `panel.rs` pattern).
fn push_sheet_rect(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<&SheetRect>,
    dst: [f32; 4],
    src: [f32; 4],
    color: [f32; 4],
) {
    let Some(sheet) = sheet else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let x0 = sheet.x as f32 + src[0];
    let y0 = sheet.y as f32 + src[1];
    let uv = [x0 / aw, y0 / ah, (x0 + src[2]) / aw, (y0 + src[3]) / ah];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, color);
}

/// A whole sheet at `dst` (source rect = the full sheet).
fn push_full_sheet(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<&SheetRect>,
    dst: [f32; 4],
    color: [f32; 4],
) {
    let Some(s) = sheet else {
        return;
    };
    push_sheet_rect(
        r,
        lookups,
        Some(s),
        dst,
        [0.0, 0.0, s.width as f32, s.height as f32],
        color,
    );
}

/// `UICreativeItemsInfiniteFilteringOptions` (`:36-73`): 11 buttons, 40 px each.
/// `active[i]` is `EntryFilterer.IsFilterActive(i)`; all false by default.
pub fn draw_filter_strip(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    layout: &Layout,
    mouse_pos: Vec2,
    active: [bool; 11],
) {
    let strip = layout.filter_strip;
    for (i, &flag) in active.iter().enumerate() {
        let button = filter_button_rect(strip, i);
        // The tab: `UIImageFramed` draws at its default `Color = Color.White`
        // (`UIImageFramed.cs:21`), unlit or hovered alike (`UIImageFramed.cs:45-49`'s
        // DrawSelf).
        let (col, row) = filter_frame_col_row(flag, in_rect(mouse_pos, button));
        push_sheet_rect(
            r,
            lookups,
            lookups.ui_infinite_tabs_b2.as_ref(),
            button,
            filter_button_src(col, row),
            [1.0; 4],
        );
        // The icon: `colorable.Color = (flag ? Color.White : (Color.White * 0.5f))`
        // (`UICreativeItemsInfiniteFilteringOptions.cs:107-110`).
        let tint = if flag {
            [1.0; 4]
        } else {
            color::tint(color::mul([255, 255, 255, 255], 0.5))
        };
        push_sheet_rect(
            r,
            lookups,
            lookups.ui_infinite_icons.as_ref(),
            filter_icon_rect(strip, i),
            filter_icon_src(FILTER_ICON_K[i]),
            tint,
        );
    }
}

/// `UIWrappedSearchBar` (`UIWrappedSearchBar.cs:76-151`) + its inner `UIPanel`.
/// `layout` supplies `search` / `search_button` / `search_box` / `search_cancel`.
///
/// The plan assumed vanilla hides the cancel icon while the box is empty; it does
/// not: `UIImageButton` in 1.4.5.8 never hides (`_visibilityInactive = 0.4f`,
/// `UIImageButton.cs:13-15`) and draws always, `White * 0.4` at rest and `White`
/// on hover (`:51`) — which is exactly what the port does. The search bar has no
/// state in P3 (`UIWrappedSearchBar.cs:155-159` only reacts to clicks).
pub fn draw_search_bar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    layout: &Layout,
    mouse_pos: Vec2,
) {
    // 1. The magnifier button (`:102-109`): the 22x22 `Button_Search_2` sheet
    //    (`:100`, `UIImageButton.cs:31-32`). `SetVisibility(1f, 1f)`
    //    (`UIWrappedSearchBar.cs:108`) means it draws at full White hovered or
    //    not (`UIImageButton.cs:51`); the `Button_Search_Border` hover frame is
    //    drawn *over* the base image, not instead of it
    //    (`UIImageButton.cs:52-55`).
    let button = layout.search_button;
    push_full_sheet(r, lookups, lookups.ui_search_icon.as_ref(), button, [1.0; 4]);
    if in_rect(mouse_pos, button) {
        push_full_sheet(
            r,
            lookups,
            lookups.ui_search_border.as_ref(),
            button,
            [1.0; 4],
        );
    }

    // 2. The text-field panel (`:110-124`): the default `PanelBackground` /
    //    `PanelBorder` sheets with `SetPadding(0)` (`:124`), both colours
    //    `ShiftBlueToCyanTheme(new Color(35, 40, 83))` (`:117-123`, the Red theme)
    //    = `(27, 56, 111, 255)`; background, then border (`UIPanel.cs:68-77`).
    let field = color::tint(color::shift_blue_to_cyan([35, 40, 83, 255]));
    let box_rect = layout.search_box;
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, box_rect, field);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, box_rect, field);

    // 3. The `Name:` placeholder (`:126-134`, `UISearchBar.cs:74-75`): the
    //    empty-box text `UI.PlayerNameSlot` (`UIWrappedSearchBar.cs:80`) in
    //    `Color.Gray` (128, 128, 128, 255) at scale 0.8 (`:126`,
    //    `UISearchBar.cs:52`), drawn through `DrawBorderString`'s 4-way shadow at
    //    spread 1.5 (`UITextPanel.cs:151-155`, `Utils.cs:2510-2519`).
    let (tx, ty) = search_text_pos(layout);
    hud_text_shadowed(r, lookups, tx, ty, 0.8, color::tint([128, 128, 128, 255]), 1.5, "Name:");

    // 4. The cancel button (`:142-150`): the 24x24 `SearchCancel` sheet. Unlike the
    //    magnifier it never calls `SetVisibility`, so it draws at the
    //    `_visibilityInactive = 0.4f` dim until hovered (`UIImageButton.cs:13-15`,
    //    `:51`). Drawn always — see the deviation note above.
    let cancel = layout.search_cancel;
    let cancel_tint = if in_rect(mouse_pos, cancel) {
        [1.0; 4]
    } else {
        color::tint(color::mul([255, 255, 255, 255], 0.4))
    };
    push_full_sheet(
        r,
        lookups,
        lookups.ui_search_cancel.as_ref(),
        cancel,
        cancel_tint,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const W: f32 = 50.0;

    #[test]
    fn layout_matches_the_source_table_at_1600x900() {
        let l = layout(Vec2::new(1600.0, 900.0), W, false);
        // `NewCraftingUI.cs:300-306`: H = 900 - 350.
        assert_eq!(l.state, [20.0, 312.0, 490.0, 550.0]);
        // `:319-325`: Top 38, H = 550 - 38.
        assert_eq!(l.panel, [20.0, 350.0, 490.0, 512.0]);
        // `UIPanel.cs:32` + `NewCraftingUI.cs:324`: 490 - 12 - 8 = 470.
        assert_eq!(l.inner, [32.0, 362.0, 470.0, 488.0]);
        // `:331-337` + `UIText.cs:138`.
        assert_eq!(l.title, [31.0, 358.0]);
        // `:338-342` + `:424`: x = 42 + 50, w = 470 - 50 - 10.
        assert_eq!(l.search, [92.0, 358.0, 410.0, 24.0]);
        // `UIWrappedSearchBar.cs:102-105`.
        assert_eq!(l.search_button, [92.0, 359.0, 22.0, 22.0]);
        // `:110-116`: w = 410 - 25, x = 92 + 410 - 385.
        assert_eq!(l.search_box, [117.0, 358.0, 385.0, 24.0]);
        // `:142-147`: a child of the box, so x is fixed regardless of the title.
        assert_eq!(l.search_cancel, [476.0, 358.0, 24.0, 24.0]);
        // `:387-390` + `UICreativeItemsInfiniteFilteringOptions.cs:40-44`.
        assert_eq!(l.filter_strip, [45.0, 316.0, 440.0, 40.0]);
        // `:352-357` + `:407-415`: H = 488 - 77, VAlign 1.
        assert_eq!(l.grid, [32.0, 439.0, 470.0, 411.0]);
        // `:359-363` + `UIHorizontalSeparator.cs:28`.
        assert_eq!(l.separator, [36.0, 439.0, 462.0, 4.0]);
        // `:367-373`.
        assert_eq!(l.list, [32.0, 446.0, 450.0, 404.0]);
        // `:375-383` + `UIScrollbar.cs:59`.
        assert_eq!(l.scrollbar, [482.0, 454.0, 20.0, 392.0]);
        // `:438`.
        assert_eq!(l.detail, [44.0, 385.0]);
        // `:484-495`.
        assert_eq!(l.grid_toggle, [42.0, 285.0]);
        assert_eq!(l.nearby_toggle, [82.0, 285.0]);
    }

    #[test]
    fn layout_tracks_the_screen_height_through_the_fills() {
        // The `-350` / `-38` / `-77` / `-7` / `-19` fills all read the parent
        // height, so a taller screen grows the panel downward from y = 350.
        let l = layout(Vec2::new(1600.0, 1080.0), W, false);
        assert_eq!(l.state, [20.0, 312.0, 490.0, 730.0]);
        assert_eq!(l.panel, [20.0, 350.0, 490.0, 692.0]);
        assert_eq!(l.inner, [32.0, 362.0, 470.0, 668.0]);
        // The title and search row hang off the panel's top: unchanged.
        assert_eq!(l.title, [31.0, 358.0]);
        assert_eq!(l.search, [92.0, 358.0, 410.0, 24.0]);
        assert_eq!(l.search_cancel, [476.0, 358.0, 24.0, 24.0]);
        // The VAlign-1 bottom of the panel moves: grid H = 668 - 77.
        assert_eq!(l.grid, [32.0, 439.0, 470.0, 591.0]);
        assert_eq!(l.separator, [36.0, 439.0, 462.0, 4.0]);
        assert_eq!(l.list, [32.0, 446.0, 450.0, 584.0]);
        assert_eq!(l.scrollbar, [482.0, 454.0, 20.0, 572.0]);
        // The detail origin and the toggles are absolute: unchanged.
        assert_eq!(l.detail, [44.0, 385.0]);
        assert_eq!(l.grid_toggle, [42.0, 285.0]);
        // The filter strip hangs off `totalContainer`'s top: unchanged.
        assert_eq!(l.filter_strip, [45.0, 316.0, 440.0, 40.0]);
    }

    #[test]
    fn difficulty_3_shifts_both_toggles_right_by_40() {
        // `NewCraftingUI.cs:486-491`: `num2 += 40` before the grid toggle and again
        // before the nearby toggle.
        let l = layout(Vec2::new(1600.0, 900.0), W, true);
        assert_eq!(l.grid_toggle, [82.0, 285.0]);
        assert_eq!(l.nearby_toggle, [122.0, 285.0]);
    }

    #[test]
    fn search_geometry_shrinks_with_the_measured_title() {
        // A different title width shifts the right-aligned bar and the box, but the
        // cancel button (a child of the box, right-aligned) stays put.
        let w = 80.0;
        let l = layout(Vec2::new(1600.0, 900.0), w, false);
        assert_eq!(l.search, [42.0 + w, 358.0, 470.0 - w - 10.0, 24.0]);
        assert_eq!(l.search_button, [42.0 + w, 359.0, 22.0, 22.0]);
        assert_eq!(l.search_box, [67.0 + w, 358.0, 435.0 - w, 24.0]);
        assert_eq!(l.search_cancel, [476.0, 358.0, 24.0, 24.0]);
    }

    #[test]
    fn indicator_bars_stack_the_fade_then_the_solid_bar() {
        // `Utils.cs:2737-2744` at the default toggle (42, 285): k = 16..=1 puts
        // `32 x 2` bars from y = 285 down to y = 300, then the `32 x 4` solid bar.
        let bars = indicator_bars(42.0, 285.0);
        assert_eq!(bars.len(), 17);
        // k = 16 (first drawn): multiplier 0.
        let (rect, col) = bars[0];
        assert_eq!(rect, [26.0, 285.0, 32.0, 2.0]);
        assert_eq!(col, color::tint(color::mul(FAVORITE_COLOR, 0.0)));
        assert_eq!(col, [0.0, 0.0, 0.0, 0.0]);
        // k = 8 (mid): m = (1 - 8/16) * 0.6 = 0.3, truncated per channel.
        let (rect, col) = bars[8];
        assert_eq!(rect, [26.0, 293.0, 32.0, 2.0]);
        assert_eq!(col, [76.0 / 255.0, 69.0 / 255.0, 20.0 / 255.0, 76.0 / 255.0]);
        // k = 1 (last fade): m = (15/16) * 0.6 = 0.5625.
        let (rect, col) = bars[15];
        assert_eq!(rect, [26.0, 300.0, 32.0, 2.0]);
        assert_eq!(
            col,
            [
                143.0 / 255.0,
                129.0 / 255.0,
                38.0 / 255.0,
                143.0 / 255.0
            ]
        );
        // The solid bar in `OurFavoriteColor`.
        let (rect, col) = bars[16];
        assert_eq!(rect, [26.0, 301.0, 32.0, 4.0]);
        assert_eq!(col, color::tint([255, 231, 69, 255]));
    }

    #[test]
    fn toggle_hover_is_a_strict_15px_square() {
        let c = Vec2::new(42.0, 285.0);
        assert!(toggle_hover(c, 42.0, 285.0));
        assert!(toggle_hover(Vec2::new(56.9, 285.0), 42.0, 285.0));
        assert!(toggle_hover(Vec2::new(28.0, 299.5), 42.0, 285.0));
        // Exactly 15 px away is outside (`CraftingUI.cs:274` uses strict `<`).
        assert!(!toggle_hover(Vec2::new(57.0, 285.0), 42.0, 285.0));
        assert!(!toggle_hover(Vec2::new(27.0, 285.0), 42.0, 285.0));
        assert!(!toggle_hover(Vec2::new(42.0, 300.0), 42.0, 285.0));
        assert!(!toggle_hover(Vec2::new(42.0, 270.0), 42.0, 285.0));
    }

    /// W4-f23 closeout item 4 — the nearby-chests toggle's frame pick
    /// (`CraftingUI.cs:340`): `num = 2 - CraftFromNearbyChests.ToInt() * 2 +
    /// hover`. ON draws the open chest (0/1), OFF the closed chest (2/3);
    /// the old `num = hover` stand-in drew the open chest even when the
    /// setting was off.
    #[test]
    fn the_nearby_toggle_frame_reads_the_live_setting() {
        let frame = |on: bool, hover: bool| 2 - (on as usize) * 2 + hover as usize;
        assert_eq!(frame(true, false), 0, "ON, idle: ChestCraft_0 (open)");
        assert_eq!(frame(true, true), 1, "ON, hovered: ChestCraft_1");
        assert_eq!(frame(false, false), 2, "OFF, idle: ChestCraft_2 (closed)");
        assert_eq!(frame(false, true), 3, "OFF, hovered: ChestCraft_3");
    }

    /// `Main.DoStatefulTickSound(ref bool, bool)` (`Main.cs:3237-3247`): the
    /// latch follows the state; only the enter edge (`false → true`) plays
    /// the cue. Held-hover and leave are silent; a re-enter plays again.
    #[test]
    fn the_stateful_tick_sound_fires_only_on_the_enter_edge() {
        let mut flag = false;
        assert!(stateful_tick_sound(&mut flag, true), "enter: cue");
        assert!(flag);
        assert!(!stateful_tick_sound(&mut flag, true), "held: silent");
        assert!(!stateful_tick_sound(&mut flag, false), "leave: silent");
        assert!(!flag);
        assert!(stateful_tick_sound(&mut flag, true), "re-enter: cue");
    }

    // ---- Task 3: the filter strip and the search bar ----

    #[test]
    fn filter_buttons_and_icons_match_the_source_table() {
        let l = layout(Vec2::new(1600.0, 900.0), W, false);
        assert_eq!(l.filter_strip, [45.0, 316.0, 440.0, 40.0]);
        for i in 0..11usize {
            // `UICreativeItemsInfiniteFilteringOptions.cs:57` + `:55-56`.
            assert_eq!(
                filter_button_rect(l.filter_strip, i),
                [45.0 + 40.0 * i as f32, 316.0, 40.0, 46.0]
            );
            // `:65-66` + the `ItemFilters.GetImage()` `VAlign = 0.5f`.
            assert_eq!(
                filter_icon_rect(l.filter_strip, i),
                [45.0 + 40.0 * i as f32 + 6.0, 316.0 + 8.0, 28.0, 30.0]
            );
        }
        // The last button ends at x = 485, inside the 490-wide panel.
        assert_eq!(filter_button_rect(l.filter_strip, 10), [445.0, 316.0, 40.0, 46.0]);
        assert_eq!(filter_icon_rect(l.filter_strip, 10), [451.0, 324.0, 28.0, 30.0]);
    }

    #[test]
    fn filter_icons_use_the_non_sequential_frame_table() {
        // `ItemFilters.GetImage`: Weapon `:206` `Frame(11)`, Armor `:241` 2,
        // Vanity `:264` 8, BuildingBlock `:98` 4, Furniture `:126` 7, Accessories
        // `:319` 1, MiscAccessories `:342` 9, Consumables `:375` 3, Tools `:183` 6,
        // Materials `:458` 10, MiscFallback `:435` 5.
        assert_eq!(FILTER_ICON_K, [0, 2, 8, 4, 7, 1, 9, 3, 6, 10, 5]);
        // Every filter maps to a distinct cell of the 11-frame sheet.
        let mut seen = [false; 11];
        for &k in FILTER_ICON_K.iter() {
            assert!(!seen[k]);
            seen[k] = true;
        }
        for k in 0..11usize {
            // `Frame(11, 1, k).OffsetSize(-2, 0)` on the 330x30 sheet.
            assert_eq!(filter_icon_src(k), [30.0 * k as f32, 0.0, 28.0, 30.0]);
        }
    }

    #[test]
    fn filter_frames_cover_all_active_hover_combinations() {
        // `:106` `SetFrame(2, 4, flag, flag * 2 + hover, -2, -2)`.
        assert_eq!(filter_frame_col_row(false, false), (0, 0));
        assert_eq!(filter_frame_col_row(false, true), (0, 1));
        assert_eq!(filter_frame_col_row(true, false), (1, 2));
        assert_eq!(filter_frame_col_row(true, true), (1, 3));
        // `Frame(2, 4)` cells on the 84x192 sheet are 42x48 (`:28-30`), trimmed to
        // 40x46 by `OffsetSize(-2, -2)` (`:55-56`).
        assert_eq!(filter_button_src(0, 0), [0.0, 0.0, 40.0, 46.0]);
        assert_eq!(filter_button_src(0, 1), [0.0, 48.0, 40.0, 46.0]);
        assert_eq!(filter_button_src(1, 2), [42.0, 96.0, 40.0, 46.0]);
        assert_eq!(filter_button_src(1, 3), [42.0, 144.0, 40.0, 46.0]);
    }

    #[test]
    fn recipe_cells_share_a_row_y_on_a_constant_44px_pitch() {
        // `UIDynamicItemCollection.cs:75-79`: integer `num / _itemsPerLine`
        // gives every recipe in a row the same Y; `:21-23` the pitch is 44;
        // `:162` `_itemsPerLine = Width / 44` — the grid fills the UIList's
        // inner list (450 wide at 1600x900), so 450 / 44 = 10 columns.
        // `GetGridParameters` (`:85-86`): startX = Center.X - 44*cols/2 =
        // (32 + 225) - 220 = 37, startY = the list's Top = 446.
        let l = layout(Vec2::new(1600.0, 900.0), W, false);
        let cols = 10usize; // 450 / 44, integer division (C# `:162`)
        let n = 25usize; // two full rows plus a partial third
        for i in 0..n {
            let cell = recipe_cell(&l, i, 0.0);
            let row = i / cols;
            let col = i % cols;
            // Y is a pure function of the row index: top + row * 44.
            assert_eq!(cell[1], 446.0 + row as f32 * 44.0, "recipe {i}");
            // X wraps left-to-right on the same pitch from the centred start.
            assert_eq!(cell[0], 37.0 + col as f32 * 44.0, "recipe {i}");
            // The hitbox is the full 44x44 cell (`:78`), not a 40x40 inset.
            assert_eq!([cell[2], cell[3]], [44.0, 44.0]);
            assert_eq!(cell[1] - 446.0, row as f32 * 44.0);
        }
        // Every recipe of the first row shares one Y, the second row the
        // next pitch — no per-iteration accumulation (the old float
        // `i as f32 / cols` drifted each icon by 44/cols = 4.4 px).
        for i in 0..cols {
            assert_eq!(recipe_cell(&l, i, 0.0)[1], 446.0);
        }
        for i in cols..2 * cols {
            assert_eq!(recipe_cell(&l, i, 0.0)[1], 490.0);
        }
        assert_eq!(recipe_cell(&l, 2 * cols, 0.0)[1], 534.0);
        // The centred rows leave a 5 px margin each side: 37 + 10*44 = 477,
        // and the list spans [32, 482] (`:85`'s integer-halved centre).
        assert_eq!(recipe_cell(&l, 9, 0.0)[0] + 44.0, 477.0);
    }

    #[test]
    fn recipe_hit_test_matches_the_drawn_cells() {
        // The click path maps through the same `recipe_cell` rects the draw
        // pass uses: the centre of cell i selects recipe i, for cells beyond
        // the first row too.
        let screen = Vec2::new(1600.0, 900.0);
        let lookups = assets::Lookups::default();
        let l = layout(screen, title_width(&lookups), false);
        let available: Vec<usize> = (0..25).collect();
        for (i, &recipe) in available.iter().enumerate() {
            let [x, y, _, _] = recipe_cell(&l, i, 0.0);
            let centre = Vec2::new(x + 22.0, y + 22.0);
            assert_eq!(
                hit_test(screen, &lookups, centre, &available, 0.0),
                Some(recipe),
                "cell {i}"
            );
        }
        // The 44x44 hitboxes tile the row with no gap (`:78` pitch == size):
        // one px left of the centred start hits nothing, and the boundary
        // between cells 0 and 1 belongs to cell 1 (exclusive left of it).
        let [x, y, _, _] = recipe_cell(&l, 0, 0.0);
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(x - 1.0, y + 20.0), &available, 0.0),
            None
        );
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(x + 44.0, y + 20.0), &available, 0.0),
            Some(available[1])
        );
    }

    /// W4-f23 wiring: `hit_test` rides the base walk's hover gate
    /// (`crafting_grid::slot_hovering`, `UIDynamicItemCollection.cs:52`) —
    /// int-truncated mouse, EXCLUSIVE right/bottom edges (the old inclusive
    /// `<=` test accepted the exact edge pixel).
    #[test]
    fn grid_hover_uses_the_base_walks_exclusive_edges() {
        let screen = Vec2::new(1600.0, 900.0);
        let lookups = assets::Lookups::default();
        let l = layout(screen, title_width(&lookups), false);
        let available: Vec<usize> = (0..25).collect();
        let [x, y, _, _] = recipe_cell(&l, 0, 0.0);
        // Just inside the right/bottom edges: still cell 0 (truncation).
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(x + 43.9, y + 43.9), &available, 0.0),
            Some(available[0])
        );
        // Exactly ON the right edge (x+44): outside cell 0 — the next cell
        // starts there, so the point lands on cell 1 (`Rectangle.Contains`).
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(x + 44.0, y + 20.0), &available, 0.0),
            Some(available[1])
        );
        // Exactly ON the bottom edge (y+44): the row below starts there.
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(x + 20.0, y + 44.0), &available, 0.0),
            Some(available[10])
        );
        // Below the last shown row: nothing (the shown list ends at row 2).
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(x + 20.0, y + 3.0 * 44.0), &available, 0.0),
            None
        );
    }

    /// W4-f23 wiring: the grid loop's per-cell machine
    /// (`NewCraftingUI.cs:55-75` via `crafting_grid::draw_slot`): a shown
    /// entry is available (`availableIndex` = its RAW `availableRecipe`
    /// position `>= 0`, `:610`) so the tint is White; the selection overlay
    /// marks ONLY the focused raw index (`OnActivate` `:904`'s
    /// `availableRecipe[focusRecipe]`), and the gamepad arm stays off
    /// (mouse-only port).
    #[test]
    fn the_grid_machine_marks_only_the_focused_recipe() {
        use super::super::crafting_grid::{RecipeEntry, available_position, draw_slot, slot_tint};
        // Raw availableRecipe = [7, 3, 12]; a filter shows only [7, 12].
        let raw = [7usize, 3, 12];
        let shown = [7usize, 12];
        // focus_recipe = 1 → availableRecipe[1] = 3 — filtered OUT of the
        // grid, so no shown cell carries the overlay (`:66` compares the
        // entry's raw recipe index against _selectedRecipeIndex).
        let selected = Some(raw[1]);
        for (i, &idx) in shown.iter().enumerate() {
            let mut e = RecipeEntry::new(idx);
            e.available_index = available_position(&raw, idx);
            e.grid_index = i as i32;
            assert!(e.available());
            assert_eq!(e.available_index, if idx == 7 { 0 } else { 2 });
            assert_eq!(slot_tint(&e), [1.0; 4], "shown entries draw White");
            let d = draw_slot(&e, selected, false, 0, false);
            assert!(!d.grid_selection_highlight, "recipe {idx} not selected");
            assert!(!d.gamepad_selected_highlight, "mouse-only");
            assert!(!d.hovered);
        }
        // With no filter shown == raw and the focused cell marks itself.
        let e = RecipeEntry::new(3);
        assert!(draw_slot(&e, Some(3), false, 0, false).grid_selection_highlight);
        // No focus (focusRecipe out of range → null, `:904`/`:614`): no overlay.
        assert!(!draw_slot(&e, None, false, 0, false).grid_selection_highlight);
    }

    /// W4-f23 doc-sweep: the grid's scroll geometry — `UpdateSize`
    /// (`UIDynamicItemCollection.cs:160-165`), `GetGridParameters`' culling
    /// window (`:81-97`) and the `UIList.cs:181` lift.
    #[test]
    fn grid_content_height_and_culling_match_the_source() {
        let l = layout(Vec2::new(1600.0, 900.0), W, false);
        // 450 / 44 = 10 columns; the list is 404 tall at 1600x900.
        assert_eq!(grid_cols(&l), 10);
        // 25 recipes = 3 rows = 132 < 404: the fill height wins, no scroll.
        assert_eq!(grid_content_height(&l, 25), l.list[3]);
        assert_eq!(clamp_scroll(50.0, &l, 25), 0.0);
        // 110 recipes = 11 rows = 484 > 404: MinHeight is the content.
        assert_eq!(grid_content_height(&l, 110), 484.0);
        assert_eq!(clamp_scroll(999.0, &l, 110), 80.0);
        // Unscrolled: start 0, end = min(110, ceil(404/44)*10 + 0 + 10) = 110
        // (`:92-96` — the +cols slack keeps the whole list).
        assert_eq!(grid_cull_range(&l, 110, 0.0), (0, 110));
        // Fully scrolled (80): num = (min(446, 366) - 446)/44 = -1 (C# int
        // division toward zero) → start 10, startY 410; end = min(110,
        // 100 + 10 + 10) = 110.
        assert_eq!(grid_cull_range(&l, 110, 80.0), (10, 110));
        // The first visible cell of row 1 sits at the lifted top + 44:
        // (446 - 80) trunc 366, -(-1)*44 → 410, row (10-10)/10 = 0 → 410.
        assert_eq!(recipe_cell(&l, 10, 80.0)[1], 410.0);
        // Row 0's cells keep their hitboxes ABOVE the list (num negative,
        // toward-zero division) — drawn clipped, not hoverable.
        assert_eq!(recipe_cell(&l, 0, 80.0)[1], 366.0);
    }

    /// W4-f23 doc-sweep: `UIList.ScrollWheel` (`:107-114`) — one notch is
    /// 120 raw units (`char_select::WHEEL_STEP`), wheel-up walks the view
    /// back, `SetView` clamps both rails.
    #[test]
    fn the_wheel_scrolls_120px_per_notch_and_clamps() {
        let l = layout(Vec2::new(1600.0, 900.0), W, false);
        // 110 recipes → 80 px of travel.
        assert_eq!(scroll_wheel(0.0, -1, &l, 110), 80.0, "down clamps at 80");
        assert_eq!(scroll_wheel(80.0, 1, &l, 110), 0.0, "up clamps at 0");
        // A short list never scrolls (the clamp range is empty).
        assert_eq!(scroll_wheel(0.0, -5, &l, 25), 0.0);
        // The wheel target: the list rect, plus the bar only while attached.
        assert!(wheel_target(&l, 110, Vec2::new(200.0, 500.0)));
        assert!(wheel_target(&l, 110, Vec2::new(490.0, 500.0)), "the bar");
        assert!(!wheel_target(&l, 25, Vec2::new(490.0, 500.0)), "detached bar");
        assert!(!wheel_target(&l, 110, Vec2::new(600.0, 500.0)), "outside");
    }

    /// W4-f23 doc-sweep: the scrolled hit-test — a cell lifted above the
    /// list's top is not hoverable (`base.IsMouseHovering` needs the
    /// pointer inside the `UIList`), and the culling window bounds the walk.
    #[test]
    fn scrolled_hit_test_stays_inside_the_list() {
        let screen = Vec2::new(1600.0, 900.0);
        let lookups = assets::Lookups::default();
        let l = layout(screen, title_width(&lookups), false);
        let available: Vec<usize> = (0..110).collect();
        // Row 0's cell 0 at scroll 80 draws at y 366 — above the list top
        // 446. Its centre (388) is outside the list: no hit.
        let c0 = recipe_cell(&l, 0, 80.0);
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(c0[0] + 22.0, c0[1] + 22.0), &available, 80.0),
            None
        );
        // Cell 10 (row 1) straddles the list top at scroll 80: it draws at
        // y 410..454, only the 446..454 sliver is inside the list. A click
        // in that sliver hits (`IsMouseHovering` needs the pointer in the
        // list; the hitbox itself spans the full 44).
        let c10 = recipe_cell(&l, 10, 80.0);
        assert_eq!(c10[1], 410.0);
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(c10[0] + 22.0, 450.0), &available, 80.0),
            Some(10)
        );
        // The cell's centre (432) is above the list top: not hoverable.
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(c10[0] + 22.0, c10[1] + 22.0), &available, 80.0),
            None
        );
        // The last row's cells still hit inside the list.
        let c109 = recipe_cell(&l, 109, 80.0);
        assert_eq!(
            hit_test(screen, &lookups, Vec2::new(c109[0] + 22.0, c109[1] + 22.0), &available, 80.0),
            Some(109)
        );
    }

    /// W4-f23 doc-sweep: `UIScrollbar.LeftMouseDown` (`:126-143`) — the
    /// handle grab latches the offset, the track press centres the handle.
    #[test]
    fn the_bar_press_grabs_the_handle_or_jumps_the_track() {
        let l = layout(Vec2::new(1600.0, 900.0), W, false);
        // Detached (25 recipes): no press.
        assert_eq!(scrollbar_press(&l, 25, 0.0, Vec2::new(490.0, 500.0)), None);
        // Attached (110 recipes): the handle at pos 0 spans the bar's top.
        let handle = panel::scrollbar_handle_at(l.scrollbar, l.list[3], 484.0, 0.0);
        let grab = Vec2::new(handle[0] + 10.0, handle[1] + 5.0);
        assert_eq!(
            scrollbar_press(&l, 110, 0.0, grab),
            Some((0.0, Some(5.0))),
            "the grab keeps mouse.Y - handle.Y"
        );
        // A track press below the handle jumps the view (no latch).
        let track = Vec2::new(handle[0] + 10.0, l.scrollbar[1] + l.scrollbar[3] - 10.0);
        let jumped = scrollbar_press(&l, 110, 0.0, track).expect("the track takes it");
        assert_eq!(jumped.1, None);
        assert!(jumped.0 > 0.0 && jumped.0 <= 80.0);
        // The drag frame follows the pointer with the latched offset:
        // the same pointer that jumped the track drags to the same value
        // when the latch is the track jump's centring offset.
        let dragged = scrollbar_drag(&l, 110, (handle[3] as i32 >> 1) as f32, track.y);
        assert_eq!(dragged, jumped.0);
    }

    #[test]
    fn search_bar_geometry_and_text_pos_match_the_source() {
        let l = layout(Vec2::new(1600.0, 900.0), W, false);
        // Already covered by `layout_matches_the_source_table_at_1600x900`; repeated
        // here so the Task 3 oracle is self-contained.
        assert_eq!(l.search, [92.0, 358.0, 410.0, 24.0]);
        assert_eq!(l.search_button, [92.0, 359.0, 22.0, 22.0]);
        assert_eq!(l.search_box, [117.0, 358.0, 385.0, 24.0]);
        assert_eq!(l.search_cancel, [476.0, 358.0, 24.0, 24.0]);
        // `UITextPanel.cs:105` + `:132-134`:
        // y = box.y + (24 - 36.8)/2 + 12 - 1.6 = box.y + 4.0.
        assert_eq!(
            search_text_pos(&l),
            (l.search_box[0] + 12.0, l.search_box[1] + 4.0)
        );
        assert_eq!(search_text_pos(&l), (129.0, 362.0));
    }
}
