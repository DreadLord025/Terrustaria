//! The achievements menu (`UIAchievementsMenu` + `UIAchievementListItem`,
//! `GameContent/UI/States/UIAchievementsMenu.cs` and
//! `GameContent/UI/Elements/UIAchievementListItem.cs`) — the scrolling list of
//! the 137 registry rows, the four category filter toggles, the title and Back
//! panels, and the per-row icon / name / description / progress-bar layout.
//!
//! Geometry follows the source: the outer container's `Width 0.8`,
//! `MaxWidth 900`/`MinWidth 700`, `Top 220` in the game menu (`OnActivate`'s
//! `Main.gameMenu` arm lowers it to 120 in-game), the `UIPanel`'s `-110`
//! height and `(12, 0, 12, 12)` inner padding, the list's `-25/-50` insets
//! with `ListPadding 5`, the `66 + 16 + 20 = 102` row height of
//! `largeForOtherLanguages = true` and the 8-per-row 64 px icons at a 66 px
//! stride (the locked copy rides +528 px).
//!
//! W4-g53 completes the screen against the C#: the `UIPanel` 9-slices and
//! byte-truncated colour pairs, the attached `UIScrollbar` (draw + drag +
//! track jump + the 120 px wheel), the list scissor, the category tooltip,
//! the `UIToggleImage` frame polarity, the row's real inner-panel origins
//! and the centre-out `ColorBar` progress bar (d81's pure halves wired in).
//!
//! The port adds the same overlay treatment the bestiary uses: the state
//! draws over the menu/world backdrop, dimmed, with its own scroll and
//! category-filter state.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text;
use super::uistate::{self, Style};
use crate::achievements::{Achievement, AchievementManager, Category};
use crate::assets::Lookups;
use crate::lang;

/// `UI.Achievements` — the title panel's key (`UIAchievementsMenu.cs`'s
/// `UITextPanel<LocalizedText>(Language.GetText("UI.Achievements"), 1f, large: true)`).
pub const TITLE_KEY: &str = "UI.Achievements";

/// `UI.Back` — the Back panel's key (`:…`'s `Language.GetText("UI.Back")`).
const BACK_KEY: &str = "UI.Back";

/// The `largeForOtherLanguages` flag is `true` for every culture the port
/// ships (`InitializePage`'s `bool flag = true`, `:…`), so the row geometry is
/// the large one: `16 + 1 * 20` on top of the 66 px base
/// (`UIAchievementListItem`'s `Height.Set(66f + num)`).
const LARGE: bool = true;

/// The row's height — `66 + (16 + 20)` for the large list item.
pub const ROW_H: f32 = 102.0;

// --- UIAchievementListItem's pure halves (W4-d81 census row 6) ------------------
//
// The draw/geometry/hover pair/sort are pre-landed (`draw_row`,
// `AchievementManager::sorted_indices` = `CompareTo` `:259-274`). These port
// the remaining stateless pieces: the small-language layout branch, the icon
// frame pick, the description's vertical scale clamp and the tracker bar's
// centre-out geometry. W4-g53 wires `tracker_bar_geometry`/`tracker_bar_colors`
// into `draw_progress` (the flat-bar stand-in is gone); the small-language
// branch stays pinned (`LARGE = true` for every shipped culture).

/// The ctor's height (`UIAchievementListItem.cs:49-53`):
/// `num = 16 + large*20`, `Height = 66 + num` — 102 large, 82 small.
#[allow(dead_code)] // the port pins LARGE = true; the small branch is ready
pub fn row_height(large: bool) -> f32 {
    66.0 + 16.0 + if large { 20.0 } else { 0.0 }
}

/// The ctor's icon offsets (`UIAchievementListItem.cs:50-51`, `:64-69`):
/// `num2 = large*6` (icon Left), `num3 = large*12` (icon Top); the borders
/// ride 4 px out on both axes (`:68-69`).
#[allow(dead_code)] // the port pins LARGE = true; the small branch is ready
pub fn icon_offsets(large: bool) -> ([f32; 2], [f32; 2]) {
    let (n2, n3) = if large { (6.0, 12.0) } else { (0.0, 0.0) };
    ([n2, n3], [n2 - 4.0, n3 - 4.0])
}

/// The icon's source rect (`UIAchievementListItem.cs:57-61`,
/// `UpdateIconFrame` `:155-169`): `x = index % 8 * 66`, `y = index / 8 * 66`,
/// 64x64; the locked copy shifts `+528` on X.
#[allow(dead_code)] // draw_icon inlines the same maths; the pure pin is testable
pub fn icon_frame_rect(icon_index: i32, locked: bool) -> [f32; 4] {
    let x = (icon_index % 8) as f32 * ICON_STRIDE + if locked { LOCKED_DX } else { 0.0 };
    let y = (icon_index / 8) as f32 * ICON_STRIDE;
    [x, y, ICON, ICON]
}

/// The description's vertical scale clamp (`UIAchievementListItem.cs:101-112`):
/// the wrap starts at scale 0.92; `num3 = 38 + (large ? 20 : 0)`; when the
/// wrapped text is taller, `scale.Y *= num3 / size.Y`. The small branch
/// re-measures the UNWRAPPED description (`:104-107`) — the caller passes the
/// matching height. Returns the clamped Y scale.
pub fn description_scale_y(wrapped_height: f32, large: bool) -> f32 {
    let cap = 38.0 + if large { 20.0 } else { 0.0 }; // `:108`
    let mut scale_y = 0.92; // `vector3` (`:101`)
    if wrapped_height > cap {
        scale_y *= cap / wrapped_height; // `:109-112`
    }
    scale_y
}

/// The tracker bar's centre-out geometry (`DrawProgressBar`,
/// `UIAchievementListItem.cs:241-256`), for the row's `Width = 80` call
/// (`:138`, `:149`): `num = Clamp(progress, 0, 1)`; the fill anchor is
/// `spot + (1, 8) + ((num - 0.5) * width, 0)` then `X--` (`:245-250`); the
/// fill is `width * num` wide, the black remainder `width * (1 - num)` from
/// the same anchor, the 2 px blip draws only when `progress != 0` (`:252`).
/// Returns `(anchor, fill_w, remainder_w, blip)`.
pub fn tracker_bar_geometry(
    spot: [f32; 2],
    width: f32,
    progress: f32,
) -> ([f32; 2], f32, f32, bool) {
    let num = progress.clamp(0.0, 1.0); // `:241`
    let mut anchor = [spot[0] + 1.0, spot[1] + 8.0]; // `:245`
    anchor[0] += (num - 0.5) * width; // `:249`
    anchor[0] -= 1.0; // `position.X--` (`:250`)
    (anchor, width * num, width * (1.0 - num), progress != 0.0)
}

/// The tracker readout's colours (`UIAchievementListItem.cs:139-148`):
/// fill `(100, 255, 100)`, track White, both `Lerp(c, Black, 0.25)` when not
/// hovered; the blip is the fill at alpha 0.5 (`:149`'s `MultiplyRGBA`).
pub fn tracker_bar_colors(hovered: bool) -> ([u8; 4], [u8; 4], [u8; 4]) {
    let dim = |c: [u8; 4]| {
        if hovered {
            c
        } else {
            super::char_creation::lerp_rgb(c, [0, 0, 0, 255], 0.25)
        }
    };
    let fill = dim([100, 255, 100, 255]); // `:139-143`
    let track = dim([255, 255, 255, 255]); // `:144-148`
    let blip = [fill[0], fill[1], fill[2], 127]; // alpha 0.5 (`:149`)
    (track, fill, blip)
}

/// `UIAchievementListItem._iconSize` = 64 (`_iconSize`).
pub const ICON: f32 = 64.0;

/// `_iconSizeWithSpace` = 66 — the icon stride on the sheet and between rows.
const ICON_STRIDE: f32 = 66.0;

/// `_iconsPerRow` = 8.
const ICONS_PER_ROW: f32 = 8.0;

/// The locked icons' column offset — `_iconFrameLocked.X += 528`
/// (`528 = 8 * 66`).
const LOCKED_DX: f32 = 528.0;

/// The 32 px toggle/icon frame size on `Achievement_Categories`, whose cells
/// ride a 34 px stride (`UIToggleImage(texture, 32, 32, new Point(34 * j, 0),
/// new Point(34 * j, 34))`).
const CAT_CELL: f32 = 34.0;
const CAT_SIZE: f32 = 32.0;

/// `InitializePage`'s `flag.ToInt() * 100` — the width bonus of the large
/// layout: `MaxWidth 900`, `MinWidth 700`, `Width 0.8`.
pub const WIDTH_BONUS: f32 = 100.0;

/// `num = _large.ToInt() * 6` — the item's icon/text left offset
/// (`UIAchievementListItem`'s `vector`).
const LARGE_LEFT: f32 = 6.0;

/// `num3 = _large.ToInt() * 12` — the icon's top offset.
const LARGE_TOP: f32 = 12.0;

/// The dark backdrop behind the menu (the port's overlay treatment — the C#
/// state draws over the live world with `IngameFancyUI`'s blur).
const BACKDROP: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

// The panel colour pairs draw through `color::mul`/`color::tint` at the draw
// sites now (the byte-truncating XNA multiply — the old float constants
// rounded differently): the menu panel `(33, 43, 79)*0.8`
// (`UIAchievementsMenu.cs:48`), the row `(26, 40, 89)*0.8` /
// `(13, 20, 44)*0.8` with the `MouseOver` pair `(46, 60, 119)` /
// `(20, 30, 56)` (`UIAchievementListItem.cs:47-48`, `:188-196`), the title
// `(73, 94, 171)` (`UIAchievementsMenu.cs:61`), the tracker colours via
// d81's `tracker_bar_colors` (`:139-149`).

/// The unlocked/locked name tints — `Color.Lerp(Silver/Gold, White, hover)`.
const SILVER: [f32; 4] = [0.753, 0.753, 0.753, 1.0];
const GOLD: [f32; 4] = [1.0, 0.843, 0.0, 1.0];
const DESCRIPTION_LOCKED: [f32; 4] = [0.663, 0.663, 0.663, 1.0]; // `Color.DarkGray`
const DESCRIPTION_OPEN: [f32; 4] = [0.753, 0.753, 0.753, 1.0]; // `Color.Silver`
const CATEGORY_ICON_TINT: [f32; 4] = [0.753, 0.753, 0.753, 1.0];

/// `Color.Gray` — the inner-panel splices' unhovered tint
/// (`UIAchievementListItem.cs:115`). XNA Gray is (128, 128, 128), unlike
/// Silver's (192, 192, 192).
const GRAY: [f32; 4] = [128.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 1.0];

/// The tracker readout's width — `float num4 = 80f`.
const BAR_W: f32 = 80.0;
/// The bar's thickness — `float num3 = 8f`.
const BAR_H: f32 = 8.0;

/// Mixes two colours (`Color.Lerp`'s linear interpolation).
fn lerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

/// `InitializePage`'s outer container (`:…`): `Width 0.8`, `MaxWidth
/// 800 + 100`, `MinWidth 600 + 100`, `Top`/`Height` by the `OnActivate`
/// `Main.gameMenu` arm, `HAlign 0.5`.
pub fn panel_rect(screen: Vec2, game_menu: bool) -> [f32; 4] {
    // `Main.gameMenu` keeps `Top 220 / Height -220`; the in-game fancy UI
    // arm drops it to `Top 120 / Height -120` (`OnActivate`, `:…`).
    let (top, height) = if game_menu {
        (220.0, Style::new(-220.0, 1.0))
    } else {
        (120.0, Style::new(-120.0, 1.0))
    };
    uistate::resolve(
        [0.0, 0.0, screen.x, screen.y], // the `UIState` root
        Style::px(0.0),                // `Left` unset
        Style::px(top),                // `Top`
        Style::new(0.0, 0.8),          // `Width.Set(0f, 0.8f)`
        height,                        // `Height`
        0.5,                           // `HAlign = 0.5f`
        0.0,                           // `VAlign` unset
        600.0 + WIDTH_BONUS,           // `MinWidth` = 700
        0.0,                           // `MinHeight` unset
        800.0 + WIDTH_BONUS,           // `MaxWidth` = 900
        [0.0; 4],                      // the container carries no padding
    )
}

/// The `UIPanel` inside the container — `Width 1`, `Height -110`
/// (`UIAchievementsMenu.cs:45-50`), which the category strip and the scrollbar
/// hang off.
pub fn inner_panel(panel: &[f32; 4]) -> [f32; 4] {
    [panel[0], panel[1], panel[2], panel[3] - 110.0]
}

/// The `UIPanel`'s padding: the ctor's `SetPadding(12)`
/// (`UIPanel.cs:32`) with `PaddingTop = 0f` overwritten
/// (`UIAchievementsMenu.cs:49`) — `(left, top, right, bottom)`.
pub const PANEL_PADDING: [f32; 4] = [12.0, 0.0, 12.0, 12.0];

/// The `UIList` — `Width -25`, `Height -50`, `Top 50`, `ListPadding 5`
/// (`UIAchievementsMenu.cs:51-56`), resolved against the panel's INNER rect
/// (`UIElement.Recalculate`, `UIElement.cs:287-288`): the 12 px side padding
/// insets X and width; the zeroed top leaves Y at the panel's own top.
pub fn list_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(inner_panel(panel), PANEL_PADDING);
    [
        inner[0],
        inner[1] + 50.0,
        inner[2] - 25.0,
        (inner[3] - 50.0).max(0.0),
    ]
}

/// The `UIScrollbar` — `Width 20` (`UIScrollbar.cs:59-60`), `Height -50`,
/// `Top 50`, `HAlign 1` (`UIAchievementsMenu.cs:81-86`), against the same
/// panel-inner rect: its right edge rides the panel's 12 px right padding.
pub fn scrollbar_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(inner_panel(panel), PANEL_PADDING);
    let h = (inner[3] - 50.0).max(0.0);
    [inner[0] + inner[2] - 20.0, inner[1] + 50.0, 20.0, h]
}

/// The category strip's container — `Width 1`, `Height 32`, `Top 10`
/// (`UIAchievementsMenu.cs:88-91`), a plain `UIElement` (no padding), so the
/// toggles resolve against the panel-inner rect directly.
pub fn category_strip_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(inner_panel(panel), PANEL_PADDING);
    [inner[0], inner[1] + 10.0, inner[2], 32.0]
}

/// The row rect at `index` inside the list — each row is `ROW_H` tall with
/// `ListPadding 5` between them, `Width 1` of the list.
pub fn row_rect(list: &[f32; 4], index: usize) -> [f32; 4] {
    [
        list[0],
        list[1] + index as f32 * (ROW_H + 5.0),
        list[2],
        ROW_H,
    ]
}

/// `MakeExitButton`-shaped Back panel: `Width (-10, 0.5)`, `Height 50`,
/// `VAlign 1`, `Top -45` (`:…`).
pub fn back_rect(panel: &[f32; 4]) -> [f32; 4] {
    let w = panel[2] * 0.5 - 10.0;
    [
        panel[0] + panel[2] * 0.5 - w * 0.5,
        panel[1] + panel[3] - 45.0 - 50.0,
        w,
        50.0,
    ]
}

/// The title panel — `UITextPanel("UI.Achievements", 1f, large: true)` with
/// `HAlign 0.5`, `Top -33` and `SetPadding(13)`.
pub fn title_rect(panel: &[f32; 4], width: f32) -> [f32; 4] {
    let h = 32.0 + 13.0 * 2.0;
    [
        panel[0] + panel[2] * 0.5 - width * 0.5,
        panel[1] - 33.0,
        width,
        h,
    ]
}

/// The four category toggles — 32 px buttons inside the strip with
/// `Left = j * 36 + 8` (`UIAchievementsMenu.cs:95-96`), resolved against the
/// strip (which itself rides the panel's 12 px left padding).
pub fn category_rects(panel: &[f32; 4]) -> [[f32; 4]; 4] {
    let strip = category_strip_rect(panel);
    std::array::from_fn(|j| {
        [
            strip[0] + (j as f32 * 36.0 + 8.0),
            strip[1],
            CAT_SIZE,
            CAT_SIZE,
        ]
    })
}

/// The `UIList` content height (`UIList.RecalculateChildren`,
/// `UIList.cs:116-128`): every row's outer height plus one `ListPadding` per
/// row (`:122`'s per-item `num2`; the single-row list drops it). N rows =
/// N*(ROW_H+5); one row = ROW_H; none = 0.
pub fn list_content_height(rows: usize) -> f32 {
    match rows {
        0 => 0.0,
        1 => ROW_H,
        n => n as f32 * (ROW_H + 5.0),
    }
}

/// Rows per viewport — the list's height over the row stride.
pub fn rows_on_screen(list: &[f32; 4]) -> usize {
    ((list[3] + 5.0) / (ROW_H + 5.0)).floor().max(1.0) as usize
}

/// The scroll range — `UIScrollbar.SetView`'s clamp upper bound
/// `maxViewSize - viewSize` (`UIScrollbar.cs:72-78`): the `UIList` content
/// height (`UIList.cs:130-137`) minus the list's own inner height.
pub fn max_scroll(rows: usize, list: &[f32; 4]) -> f32 {
    (list_content_height(rows) - list[3]).max(0.0)
}

/// `UIList.ScrollWheel` (`UIList.cs:107-114`): `_scrollbar.ViewPosition -=
/// evt.ScrollWheelValue` — one detent is 120 raw units
/// (`char_select::WHEEL_STEP`, `PlayerInput.cs:898-899`), clamped by
/// `SetView`. `delta` is in notches (positive = wheel up).
pub fn scroll_wheel(scroll: f32, delta: i32, rows: usize, list: &[f32; 4]) -> f32 {
    (scroll - delta as f32 * super::char_select::WHEEL_STEP)
        .clamp(0.0, max_scroll(rows, list))
}

/// What a press on the scrollbar does (`UIScrollbar.LeftMouseDown`,
/// `UIScrollbar.cs:126-143`): `(new view position, drag latch)` — the same
/// shape `char_select::scrollbar_press` ports. `Some(offset)` is the
/// `_dragYOffset = mouse.Y - handle.Y` grab (`:134-135`); `None` inside is
/// the track jump (`:137-141`); `None` as a whole means the press missed
/// the bar.
pub fn scrollbar_press(
    panel: &[f32; 4],
    rows: usize,
    scroll: f32,
    mouse: Vec2,
) -> Option<(f32, Option<f32>)> {
    let bar = scrollbar_rect(panel);
    let list = list_rect(panel);
    let content = list_content_height(rows);
    if !super::panel::scrollbar_can_scroll(list[3].min(content), content)
        || !in_rect(bar, mouse)
    {
        return None;
    }
    let handle = super::panel::scrollbar_handle_at(bar, list[3], content, scroll);
    if in_rect(handle, mouse) {
        return Some((scroll, Some(mouse.y - handle[1])));
    }
    Some((
        super::panel::scrollbar_jump_value(bar, list[3], content, handle[3], mouse.y),
        None,
    ))
}

/// The drag frame (`UIScrollbar.DrawSelf`, `UIScrollbar.cs:109-112`).
pub fn scrollbar_drag(panel: &[f32; 4], rows: usize, drag_offset: f32, mouse_y: f32) -> f32 {
    let bar = scrollbar_rect(panel);
    let list = list_rect(panel);
    super::panel::scrollbar_drag_value(
        bar,
        list[3],
        list_content_height(rows),
        drag_offset,
        mouse_y,
    )
}

/// `FilterList` (`UIAchievementsMenu.cs:163-175`): a row shows when its
/// category's toggle is on. Every registry row carries one of the four
/// toggle categories — `GAIN_TORCH_GODS_FAVOR`, the one row the
/// initializer's category pass never registers, keeps the CLR enum-field
/// default `(AchievementCategory)0 = Slayer`, NOT `None = -1`
/// (`Achievement.cs:25`'s unset `_category` + `AchievementCategory.cs:5-6`),
/// so vanilla's `_categoryButtons[(int)Category]` never indexes -1 and
/// never throws (W4-g53 census correction).
pub fn filter_visible(categories: &[bool; 4], category: Category) -> bool {
    match category.index() {
        // No registry row is None; the defensive arm mirrors the C#'s
        // out-of-range throw by hiding the row rather than crashing.
        -1 => false,
        i => categories[i as usize],
    }
}

/// Which row (in registry order) the pointer is over, by the row lattice.
pub fn row_at(list: &[f32; 4], scroll: f32, mouse: Vec2, rows: usize) -> Option<usize> {
    if mouse.x < list[0] || mouse.x > list[0] + list[2] {
        return None;
    }
    if mouse.y < list[1] || mouse.y > list[1] + list[3] {
        return None;
    }
    let idx = ((mouse.y - list[1] + scroll) / (ROW_H + 5.0)) as usize;
    (idx < rows).then_some(idx)
}

/// The category toggle under the pointer (`FilterList`'s click arm).
pub fn category_at(panel: &[f32; 4], mouse: Vec2) -> Option<usize> {
    category_rects(panel)
        .iter()
        .position(|r| in_rect(*r, mouse))
}

/// The Back panel under the pointer.
pub fn back_hit(panel: &[f32; 4], mouse: Vec2) -> bool {
    in_rect(back_rect(panel), mouse)
}

fn in_rect(r: [f32; 4], p: Vec2) -> bool {
    p.x >= r[0] && p.x <= r[0] + r[2] && p.y >= r[1] && p.y <= r[1] + r[3]
}

/// `Achievements.<NAME>_Name` (`Achievement.FriendlyName`'s key).
pub fn name_key(name: &str) -> String {
    format!("Achievements.{name}_Name")
}

/// `Achievements.<NAME>_Description` (`Achievement.Description`'s key).
pub fn desc_key(name: &str) -> String {
    format!("Achievements.{name}_Description")
}

/// The scrolling category-filtered list — `InitializePage`'s body rebuilt
/// for the draw, with the same `CompareTo` order the `UIList` applies.
pub fn visible_rows(manager: &AchievementManager, categories: &[bool; 4]) -> Vec<usize> {
    manager
        .sorted_indices()
        .into_iter()
        .filter(|&i| filter_visible(categories, manager.list[i].category))
        .collect()
}

/// `UIAchievementsMenu.Draw`, with the port's overlay treatment.
/// `mouse_text_color` is `Main.mouseTextColor` (the cycling 225±30 grey the
/// category tooltip draws with, `UIAchievementsMenu.cs:132`); `dragging` is
/// the scrollbar's `_isDragging` latch (`UIScrollbar.cs:122`'s handle-tint
/// gate — the hover half is re-derived from the pointer here, the sound
/// edge ticks in `App::achievements_hover_tick`).
#[allow(clippy::too_many_arguments)]
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    manager: &AchievementManager,
    scroll: f32,
    categories: &[bool; 4],
    mouse: Vec2,
    game_menu: bool,
    mouse_text_color: u8,
    dragging: bool,
) {
    let panel = panel_rect(screen, game_menu);
    r.push_ui_sprite([0.0, 0.0], [screen.x, screen.y], r.white_uv(), BACKDROP);

    // The `UIPanel` body — `BackgroundColor = (33, 43, 79) * 0.8f`
    // (`UIAchievementsMenu.cs:48`) through the 9-slice, plus the `UIPanel`
    // default black border (`UIPanel.cs:18`); the byte multiply truncates
    // exactly (`color::mul`).
    let inner = inner_panel(&panel);
    let bg = super::color::mul([33, 43, 79, 255], 0.8);
    super::panel::draw_panel(r, lookups, lookups.ui_panel_bg, inner, super::color::tint(bg));
    super::panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        inner,
        super::color::tint(uistate::TEXT_PANEL_BORDER),
    );

    draw_title(r, lookups, &panel);
    draw_categories(r, lookups, &panel, categories, mouse);
    draw_back(r, lookups, &panel, mouse);
    let rows = visible_rows(manager, categories);
    draw_list(r, lookups, &panel, manager, &rows, scroll, mouse);
    // The `UIScrollbar` (`InitializePage`'s `SetView(100f, 1000f)` placeholder
    // is overwritten by `SetScrollbar`'s `UpdateScrollbar`, `UIList.cs:139-143`:
    // view = the list height, max = the content height). `AutoHide` stays
    // false (`UIScrollbar.cs:29`), so the bar draws even on a short list.
    // The handle tint is the live `dragging || hovering` pair
    // (`UIScrollbar.cs:122`); the hover latch + sound-12 edge tick in
    // `App::achievements_hover_tick` (`:113-120`).
    let bar = scrollbar_rect(&panel);
    let view = list_rect(&panel)[3];
    let content = list_content_height(rows.len());
    let handle_hover = super::panel::scrollbar_hover_tick(bar, view, content, scroll, mouse, false).0;
    super::panel::draw_scrollbar_handle_tint(
        r,
        lookups,
        bar,
        view,
        content,
        scroll,
        false,
        super::panel::scrollbar_handle_tint(dragging, handle_hover),
    );
    draw_category_tooltip(r, lookups, screen, &panel, mouse, mouse_text_color);
}

/// `UIAchievementsMenu.Draw`'s tooltip arm (`:108-135`): the FIRST hovered
/// category button (the loop `break`s) draws its `Achievements.*Category`
/// label at `mouse + (16, 16)`, clamped `Y <= screenH - 30` and — when it
/// would run off the right edge — `X = screenW - 460` (the C#'s literal,
/// `:130`). `Utils.DrawBorderStringFourWay` through the Mouse font in
/// `Main.mouseTextColor` grey with the black ±2 px ring (`Utils.cs:2464-2496`).
fn draw_category_tooltip(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    panel: &[f32; 4],
    mouse: Vec2,
    mouse_text_color: u8,
) {
    let Some(j) = category_at(panel, mouse) else {
        return;
    };
    let label = lang::text(Category::FILTERS[j].tooltip_key());
    let w = text::ui_font_width(lookups, &label, 1.0).unwrap_or(100.0);
    let mut pos = mouse + Vec2::splat(16.0); // `:123`
    if pos.y > screen.y - 30.0 {
        pos.y = screen.y - 30.0; // `:124-127`
    }
    if pos.x > screen.x - w {
        pos.x = screen.x - 460.0; // `:128-131`
    }
    let c = mouse_text_color as f32 / 255.0;
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Ui,
        pos.x,
        pos.y,
        1.0,
        &[[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]],
        [c, c, c, c], // `new Color(mtc, mtc, mtc, mtc)` — alpha rides too (`:132`)
        &label,
    );
}

/// The `UITextPanel` title — `UI.Achievements` at scale 1.0 through the
/// large (Death) font, the panel's `(73, 94, 171)` background and black
/// border, `SetPadding(13)` (`UIAchievementsMenu.cs:57-61`).
fn draw_title(r: &mut SpriteRenderer, lookups: &Lookups, panel: &[f32; 4]) {
    let label = lang::text(TITLE_KEY);
    // `SetText`'s MinWidth = measured.X + PaddingLeft + PaddingRight
    // (`UITextPanel.cs:132-133`) — the panel is the label plus the 13 px
    // padding on both sides.
    let w = label_width(lookups, &label) + 13.0 * 2.0;
    let rect = title_rect(panel, w);
    super::uistate::draw_text_panel(
        r,
        lookups,
        rect,
        &title_style(),
        &label,
        label_width(lookups, &label),
        false,
    );
}

/// The large-title style — `UITextPanel`'s `large: true` scale-1 form with
/// the ctor's `BackgroundColor = new Color(73, 94, 171)` (`:61`) over the
/// `UIPanel` default black border.
fn title_style() -> uistate::TextPanelStyle {
    uistate::TextPanelStyle {
        text_scale: 1.0,
        large: true,
        padding: 13.0,
        bg: [73, 94, 171, 255],
        border: uistate::TEXT_PANEL_BORDER,
    }
}

/// `UITextPanel.MeasureString` through the Death font (`UITextPanel.cs:126-131`).
fn label_width(lookups: &Lookups, label: &str) -> f32 {
    match &lookups.death_font {
        Some(f) => text::font_width(f, label, 1.0),
        None => 100.0,
    }
}

/// The four `UIToggleImage` filter buttons (`Draw`'s category switch,
/// `UIAchievementsMenu.cs:93-101`). The ctor's `Point(34*j, 0)` lands in
/// `onTextureOffset` and `Point(34*j, 34)` in `offTextureOffset`
/// (`UIToggleImage.cs:26-36` — the parameter order is `(on, off)` while the
/// call site passes them row-0-first), so the ON frame is the sheet's TOP
/// row. `DrawSelf` tints `White` on hover, `Silver` otherwise (`:53`).
fn draw_categories(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    panel: &[f32; 4],
    categories: &[bool; 4],
    mouse: Vec2,
) {
    for (j, rect) in category_rects(panel).iter().enumerate() {
        let hovered = in_rect(*rect, mouse);
        let frame_y = if categories[j] { 0.0 } else { CAT_CELL };
        draw_category_frame(
            r,
            lookups,
            j,
            frame_y,
            [rect[0], rect[1], CAT_SIZE, CAT_SIZE],
            if hovered { [1.0; 4] } else { SILVER },
        );
    }
}

/// One 32x32 frame of `Achievement_Categories` at column `col`, row 0 or 1.
fn draw_category_frame(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    col: usize,
    frame_y: f32,
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let Some(sheet) = lookups.ach_categories else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let x0 = sheet.x as f32 + col as f32 * CAT_CELL;
    let y0 = sheet.y as f32 + frame_y;
    let uv = [
        x0 / aw,
        y0 / ah,
        (x0 + CAT_SIZE) / aw,
        (y0 + CAT_SIZE) / ah,
    ];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

/// The row's category icon — `Frame(4, 2, category)` at scale 0.5
/// (`UIAchievementListItem.cs:121`): the source cell is the sheet's
/// `width/4 x height/2` quarter (`Utils.cs:1344-1348`), drawn at half size.
fn draw_category_frame_scaled(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    col: usize,
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let Some(sheet) = lookups.ach_categories else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let cw = (sheet.width / 4) as f32;
    let ch = (sheet.height / 2) as f32;
    let x0 = sheet.x as f32 + col as f32 * cw;
    let y0 = sheet.y as f32;
    let uv = [x0 / aw, y0 / ah, (x0 + cw) / aw, (y0 + ch) / ah];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

/// The Back panel — `UITextPanel("UI.Back", 0.7f, large: true)`
/// (`UIAchievementsMenu.cs:63-73`) with this state's OWN
/// `FadedMouseOver`/`FadedMouseOut` pair (`:150-161`): hover
/// `(73, 94, 171)` / `Colors.FancyUIFatButtonMouseOver`, idle
/// `(63, 82, 151) * 0.8f` / Black — note the 0.8, not the shared
/// fat-button style's 0.7.
fn draw_back(r: &mut SpriteRenderer, lookups: &Lookups, panel: &[f32; 4], mouse: Vec2) {
    let rect = back_rect(panel);
    let label = lang::text(BACK_KEY);
    let mut style = super::cards::fat_button_style();
    style.bg = super::color::mul([63, 82, 151, 255], 0.8); // `:159`
    // `SetText` measures through the LARGE (Death) font (`UITextPanel.cs:80-90`).
    let text_w = text::large_text_width(r, lookups, &label) * style.text_scale;
    // `FadedMouseOver` paints `(73, 94, 171)` with the fat-button hover
    // border; the draw helper's hover flag carries the same pair.
    super::uistate::draw_text_panel(
        r,
        lookups,
        rect,
        &style,
        &label,
        text_w,
        back_hit(panel, mouse),
    );
}

/// The filtered, sorted list (`FilterList` + `UpdateOrder`): the rows ride
/// `_innerList.Top = -ViewPosition` (`UIList.DrawSelf`, `UIList.cs:177-184`)
/// under the list's scissor (`OverflowHidden = true`, `UIList.cs:59` —
/// `UIInnerList.GetViewCullingArea` returns the list rect, `:34-37`); only
/// the rows whose boxes overlap the viewport draw (`UIInnerList.
/// DrawChildren`'s AABB gate, `UIList.cs:21-32`).
fn draw_list(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    panel: &[f32; 4],
    manager: &AchievementManager,
    rows: &[usize],
    scroll: f32,
    mouse: Vec2,
) {
    let list = list_rect(panel);
    r.push_clip(list);
    let first = ((scroll / (ROW_H + 5.0)).floor()).max(0.0) as usize;
    let count = rows_on_screen(&list);
    for slot in 0..=count {
        let idx = match rows.get(first + slot) {
            Some(&i) => i,
            None => break,
        };
        let rect = row_rect(&list, first + slot);
        let shifted = [rect[0], rect[1] - scroll, rect[2], rect[3]];
        let a = &manager.list[idx];
        // Hover is the row's own box (`IsMouseHovering`) intersected with the
        // viewport — a row half-scrolled-out only hovers on its visible half.
        let hover = in_rect(shifted, mouse) && in_rect(list, mouse);
        draw_row(r, lookups, &shifted, a, hover);
    }
    r.pop_clip();
}

/// One `UIAchievementListItem` — the panel, the icon + border, the category
/// icon, the name, the description and the tracker's progress bar.
fn draw_row(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: &[f32; 4],
    a: &Achievement,
    hover: bool,
) {
    let locked = !a.completed;
    // The item panel — `UIAchievementListItem` IS a `UIPanel`, so the
    // background/border pair draws through the 9-slice: ctor
    // `(26, 40, 89)*0.8` / `(13, 20, 44)*0.8` (`UIAchievementListItem.cs:47-48`),
    // `MouseOver` `(46, 60, 119)` / `(20, 30, 56)` (`:188-189`); the byte
    // multiplies truncate exactly.
    let (bg, border) = if hover {
        ([46, 60, 119, 255], [20, 30, 56, 255])
    } else {
        (
            super::color::mul([26, 40, 89, 255], 0.8),
            super::color::mul([13, 20, 44, 255], 0.8),
        )
    };
    super::panel::draw_panel(r, lookups, lookups.ui_panel_bg, *rect, super::color::tint(bg));
    super::panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        *rect,
        super::color::tint(border),
    );

    // `PaddingTop 8` / `PaddingLeft 9` (`:55-56`).
    let ix = rect[0] + 9.0 + LARGE_LEFT;
    let iy = rect[1] + 8.0 + LARGE_TOP;

    // The icon — locked frame until the row completes (`UpdateIconFrame`).
    draw_icon(r, lookups, ix, iy, a.icon, locked);

    // The border, 4 px out on both axes (`:68-69`). `UIImage.Color` stays
    // `Color.White` (`UIImage.cs:20`) — no hover tint on this element.
    if let Some(sheet) = lookups.ach_borders {
        super::push_whole(
            r,
            lookups,
            &sheet,
            [ix - 4.0, iy - 4.0, sheet.width as f32, sheet.height as f32],
            [1.0; 4],
        );
    }

    let border_w = lookups.ach_borders.map_or(72.0, |s| s.width as f32);
    // `vector2 = border dimensions' right edge + 7` (`:92`) and
    // `num2 = innerWidth - borderWidth + 1 - num*2` (`:99`); the inner width
    // is the row minus the 9 px left + 12 px right `UIPanel` padding
    // (`UIPanel.cs:32` — the ctor only overwrites Top/Left). The borders'
    // Left is `-4 + num2` off the inner X (`:68`).
    let inner_x = rect[0] + 9.0;
    let inner_y = rect[1] + 8.0;
    let inner_w = rect[2] - 9.0 - 12.0;
    let text_x = inner_x + (LARGE_LEFT - 4.0) + border_w + 7.0 + LARGE_LEFT;
    let text_w = inner_w - border_w + 1.0 - LARGE_LEFT * 2.0;

    // `color = IsMouseHovering ? White : Gray` (`:115`) — the inner-panel
    // splices' tint. `Color.Gray` is (128, 128, 128), NOT Silver's 192.
    let panel_tint = if hover { [1.0; 4] } else { GRAY };

    // The name row — `vector4 = vector2 - UnitY*2 + vector` (`:116`) with
    // `vector = (num, 0)` (`:87`): vector2.Y is the INNER top (PaddingTop 8),
    // so the splice rides innerY - 2, above the icon's own +12 top.
    let name_y = inner_y - 2.0;
    draw_inner_panel_top(r, lookups, text_x, name_y, text_w, panel_tint);
    // The category icon — `Frame(4, 2, category)` at scale 0.5 (`:121`):
    // a 34x34 source cell drawn 17x17. `Frame(4, 2, i)` reads column i of
    // row 0 (`Utils.cs:1344-1348` — the third arg is frameX).
    let cat_x = text_x + 4.0;
    let cat_y = name_y + 2.0;
    draw_category_frame_scaled(
        r,
        lookups,
        a.category.index().max(0) as usize,
        [cat_x, cat_y, CAT_CELL * 0.5, CAT_CELL * 0.5],
        if hover { [1.0; 4] } else { CATEGORY_ICON_TINT },
    );
    // `vector4.X += 4; vector4.X += 17` (`:122-123`)
    let name_x = cat_x + 4.0 + 17.0;
    let name = lang::text(&name_key(a.name));
    let name_color = lerp(if locked { SILVER } else { GOLD }, [1.0; 4], if hover { 0.5 } else { 0.0 });
    push_clipped_text(
        r,
        lookups,
        name_x,
        cat_y,
        0.85,
        name_color,
        &name,
        text_w - (name_x - text_x),
    );

    // The description row — `position = vector2 + UnitY*27 + vector`
    // (`:126`), the bottom splice at that origin, then the text at +8/+4
    // (`:128-130`). Same vector2.Y base as the name row (the vector's Y is 0).
    let desc_panel_y = inner_y + 27.0;
    draw_inner_panel_bottom(r, lookups, text_x, desc_panel_y, text_w, panel_tint);
    let desc_color = lerp(if locked { DESCRIPTION_LOCKED } else { DESCRIPTION_OPEN }, [1.0; 4], if hover { 1.0 } else { 0.0 });
    let desc = lang::text(&desc_key(a.name));
    // `CreateWrappedText(Description, (num2 - 20) * (1/0.92), culture)`
    // (`:102`) through d121's ReLogic wrap port, then the `GetStringSize`
    // Y-clamp (`:103-112`), then the wrapped string's rows at +8/+4
    // (`:128-130`) — the horizontal shrink-to-fit stand-in is gone.
    let (wrapped, scale_y) = description_layout(lookups, &desc, text_w);
    draw_description(r, lookups, text_x + 8.0, desc_panel_y + 4.0, scale_y, desc_color, &wrapped);

    // The tracker bar — only while locked and only with a tracker
    // (`flag = (Item1 != 0 || Item2 != 0) && _locked`, `:93-98`). The anchor
    // is `vector4` AFTER the name walk (`:133`): the name origin + num2 on X
    // (the `:125` `-= 17` cancels the second `+= 17`), +1 on Y.
    if let Some((value, max)) = a.tracker_values() {
        if locked && (value != 0 || max != 0) {
            draw_progress(r, lookups, cat_x + 4.0 + text_w, cat_y + 1.0, value, max, name_color, hover);
        }
    }
}

/// The 64x64 icon of `Images/UI/Achievements`, column
/// `icon % 8`, row `icon / 8`, shifted +528 px while locked.
fn draw_icon(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    x: f32,
    y: f32,
    icon: u16,
    locked: bool,
) {
    let Some(sheet) = lookups.ach_icons else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let col = icon as f32 % ICONS_PER_ROW;
    let row = (icon as f32 / ICONS_PER_ROW).floor();
    let x0 = sheet.x as f32 + col * ICON_STRIDE + if locked { LOCKED_DX } else { 0.0 };
    let y0 = sheet.y as f32 + row * ICON_STRIDE;
    let uv = [
        x0 / aw,
        y0 / ah,
        (x0 + ICON) / aw,
        (y0 + ICON) / ah,
    ];
    r.push_ui_sprite([x, y], [ICON, ICON], uv, [1.0; 4]);
}

/// `DrawPanelTop` (`UIAchievementListItem.cs:171-176`) — three 2 px-wide
/// splices of `Achievement_InnerPanelTop`, the middle stretched to
/// `(width - 4) / 2`, all tinted by the hover `color` (`:115`).
fn draw_inner_panel_top(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    x: f32,
    y: f32,
    width: f32,
    tint: [f32; 4],
) {
    let Some(sheet) = lookups.ach_inner_top else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let h = sheet.height as f32;
    let base_y = sheet.y as f32;
    // The left cap.
    splice(r, lookups, sheet.x as f32, base_y, 2.0, h, x, y, 2.0, h, aw, ah, tint);
    // The stretched middle: `(width - 4) / 2` horizontally.
    let mid_w = (width - 4.0) / 2.0;
    splice(
        r,
        lookups,
        sheet.x as f32 + 2.0,
        base_y,
        2.0,
        h,
        x + 2.0,
        y,
        mid_w.max(0.0),
        h,
        aw,
        ah,
        tint,
    );
    // The right cap.
    splice(
        r,
        lookups,
        sheet.x as f32 + 4.0,
        base_y,
        2.0,
        h,
        x + width - 2.0,
        y,
        2.0,
        h,
        aw,
        ah,
        tint,
    );
}

/// `DrawPanelBottom` (`UIAchievementListItem.cs:178-183`) — the 6/7/6 px
/// splices of `Achievement_InnerPanelBottom` (the `_Large` variant while
/// large, `:72-79`), tinted by the same hover `color`.
fn draw_inner_panel_bottom(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    x: f32,
    y: f32,
    width: f32,
    tint: [f32; 4],
) {
    let Some(sheet) = lookups.ach_inner_bottom_large.or(lookups.ach_inner_bottom) else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let h = sheet.height as f32;
    let base_y = sheet.y as f32;
    // Left cap: `Rectangle(0, 0, 6, h)`.
    splice(
        r,
        lookups,
        sheet.x as f32,
        base_y,
        6.0,
        h,
        x,
        y,
        6.0,
        h,
        aw,
        ah,
        tint,
    );
    // Middle: `Rectangle(6, 0, 7, h)` stretched `(width - 12) / 7`.
    let mid_w = (width - 12.0) / 7.0;
    splice(
        r,
        lookups,
        sheet.x as f32 + 6.0,
        base_y,
        7.0,
        h,
        x + 6.0,
        y,
        mid_w.max(0.0),
        h,
        aw,
        ah,
        tint,
    );
    // Right cap: `Rectangle(13, 0, 6, h)`.
    splice(
        r,
        lookups,
        sheet.x as f32 + 13.0,
        base_y,
        6.0,
        h,
        x + width - 6.0,
        y,
        6.0,
        h,
        aw,
        ah,
        tint,
    );
}

/// One source-rect → destination-rect sprite.
#[allow(clippy::too_many_arguments)]
fn splice(
    r: &mut SpriteRenderer,
    _lookups: &Lookups,
    sx: f32,
    sy: f32,
    sw: f32,
    sh: f32,
    dx: f32,
    dy: f32,
    dw: f32,
    dh: f32,
    aw: f32,
    ah: f32,
    tint: [f32; 4],
) {
    let uv = [sx / aw, sy / ah, (sx + sw) / aw, (sy + sh) / ah];
    r.push_ui_sprite([dx, dy], [dw, dh], uv, tint);
}

/// `DrawProgressBar` with `Width = 80` (`UIAchievementListItem.cs:131-151`,
/// `:224-257`), built on d81's pure `tracker_bar_geometry`/`tracker_bar_colors`
/// halves: the `ColorBar` track (centre-anchored stretched body + the 5/4 px
/// end caps, `:246-248`), then the centre-out `MagicPixel` fill, the 2 px
/// blip and the black remainder (`:249-256`), then the `value / max` readout
/// at scale 0.75 in the NAME colour (`baseColor`, `:151` — not white).
/// `anchor` is `vector5` (`:133`); the flat-bar stand-in is gone.
#[allow(clippy::too_many_arguments)]
fn draw_progress(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    anchor_x: f32,
    anchor_y: f32,
    value: i32,
    max: i32,
    name_color: [f32; 4],
    hover: bool,
) {
    let progress = if max != 0 {
        value as f32 / max as f32
    } else {
        0.0
    };
    let readout = format!("{value}/{max}");
    let readout_w = text::hud_item_text_width(r, lookups, &readout, 0.75);
    // `spot = vector5 - UnitX * num4 * 0.7f` (`:149`).
    let spot = [anchor_x - BAR_W * 0.7, anchor_y];
    let (track, fill, blip) = tracker_bar_colors(hover);
    let (track, fill, blip) = (
        super::color::tint(track),
        super::color::tint(fill),
        super::color::tint(blip),
    );
    // The `ColorBar` track: `num4 = Width / 169` scale, the body drawn from
    // `Rectangle(5, 0, W-9, H)` with origin (84.5, 0) — i.e. centred on
    // `spot.X` — and the 5 px / 4 px caps at `spot.X - num4*84.5 - 5` and
    // `spot.X + num4*84.5` (`:244-248`). At Width 80 the half-span is
    // `80/169*84.5 = 40`, so the caps ride the bar's own ends.
    if let Some(sheet) = lookups.ui_color_bar {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let (sw, sh) = (sheet.width as f32, sheet.height as f32);
        let half = BAR_W / 169.0 * 84.5;
        let body_w = (sw - 9.0) * (BAR_W / 169.0);
        let sx = sheet.x as f32;
        let sy = sheet.y as f32;
        // The stretched body (`:246`).
        splice(r, lookups, sx + 5.0, sy, sw - 9.0, sh, spot[0] - half, spot[1], body_w, sh, aw, ah, track);
        // The left cap (`:247`).
        splice(r, lookups, sx, sy, 5.0, sh, spot[0] - half - 5.0, spot[1], 5.0, sh, aw, ah, track);
        // The right cap (`:248`).
        splice(r, lookups, sx + sw - 4.0, sy, 4.0, sh, spot[0] + half, spot[1], 4.0, sh, aw, ah, track);
    }
    // The fill/blip/remainder from the pure geometry (`:249-256`): the
    // anchor is the progress TIP; the fill extends LEFT of it (origin
    // (1, 0.5)), the black remainder RIGHT (origin (0, 0.5)), both 8 px
    // tall centred on the anchor's Y.
    let (tip, fill_w, rest_w, blip_on) = tracker_bar_geometry(spot, BAR_W, progress);
    let y = tip[1] - BAR_H * 0.5;
    r.push_ui_sprite([tip[0] - fill_w, y], [fill_w, BAR_H], r.white_uv(), fill);
    if blip_on {
        // The 2 px blip rides the tip with origin (1, 0.5) (`:254`).
        r.push_ui_sprite([tip[0] - 2.0, y], [2.0, BAR_H], r.white_uv(), blip);
    }
    r.push_ui_sprite([tip[0], y], [rest_w, BAR_H], r.white_uv(), [0.0, 0.0, 0.0, 1.0]);
    // The readout at `vector5.X -= num4 * 1.4f + stringSize2.X` (`:150-151`).
    let x = anchor_x - BAR_W * 1.4 - readout_w;
    text::hud_item_text(r, lookups, x, anchor_y, 0.75, name_color, &readout);
}

/// The name row's clamp (`ChatManager.DrawColorCodedStringWithShadow(…,
/// num2)`'s snippet walk, `UIAchievementListItem.cs:124`): names carry no
/// '\n' and no chat tags, so the port draws the raw string at the source
/// scale, shrunk to the row's text width when it overflows (the snippet
/// layout's word-wrap arm is unreachable for the short registry names).
fn push_clipped_text(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
    max_w: f32,
) {
    if max_w <= 0.0 {
        return;
    }
    let w = text::hud_item_text_width(r, lookups, text, scale);
    if w > max_w {
        // Shrink to fit rather than overflowing the row — the name-row
        // stand-in only; the description wraps through `description_layout`.
        let scale = (scale * max_w / w).max(0.4);
        text::hud_item_text(r, lookups, x, y, scale, color, text);
        return;
    }
    text::hud_item_text(r, lookups, x, y, scale, color, text);
}

/// The description's wrap + vertical clamp (`UIAchievementListItem.cs:
/// 101-112`): `text = CreateWrappedText(Description, (num2 - 20) *
/// (1 / 0.92), culture)` (`:102` — the wrap width is the TEXT width minus
/// 20, divided by the X scale so the wrapped box fits at 0.92), measured
/// through `ChatManager.GetStringSize(ItemStack, text, (0.92, 0.92), num2)`
/// (`:103`); the large layout keeps the wrapped measure, the small branch
/// re-measures the UNWRAPPED description (`:104-107`, pinned out — `LARGE =
/// true`). `description_scale_y` clamps Y at the 38+20 cap (`:108-112`).
/// Returns `(wrapped text, clamped Y scale)`.
pub fn description_layout(lookups: &Lookups, desc: &str, text_w: f32) -> (String, f32) {
    // `(num2 - 20f) * (1f / vector3.X)` (`:102`).
    let wrap_w = (text_w - 20.0) * (1.0 / 0.92);
    let wrapped = super::text_wrap::create_wrapped_text(
        lookups.item_font.as_ref(),
        desc,
        wrap_w,
        crate::lang::active_culture(),
    );
    // `GetStringSize(font, text, vector3, num2)` — one snippet, no tags:
    // the box is `MeasureString(wrapped) * scale` (the maxWidth re-wrap in
    // `LayoutSnippets` never fires — the wrap already fits num2). The
    // measure's height floor stands in as the drawn glyph height (d121's
    // recorded `Padding.Height` approximation).
    let (_, h) = super::text_wrap::measure_string(lookups.item_font.as_ref(), &wrapped);
    let scale_y = description_scale_y(h * 0.92, LARGE);
    (wrapped, scale_y)
}

/// The wrapped description's draw (`UIAchievementListItem.cs:130`'s
/// `DrawColorCodedStringWithShadow(ItemStack, text, position, baseColor2,
/// …, vector3)`): the '\n'-separated rows at X scale 0.92, Y scale the
/// clamp, stepping `LineSpacing * scale.Y` per row (`DynamicSpriteFont.
/// DrawString`'s newline arm). The shadow is the four ±2 px black passes
/// (`ChatManager.ShadowDirections`, `ChatManager.cs:376-382` — the port's
/// existing bordered-string convention stands in for the parse-free fast
/// path's ±2 ring).
fn draw_description(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    x: f32,
    y: f32,
    scale_y: f32,
    color: [f32; 4],
    wrapped: &str,
) {
    match lookups.item_font.as_ref() {
        Some(f) => {
            // The shadow passes (`ChatManager.cs:338-342`).
            for [dx, dy] in [[-2.0, 0.0], [2.0, 0.0], [0.0, -2.0], [0.0, 2.0]] {
                text::push_font_text_xy(
                    r,
                    f,
                    x + dx,
                    y + dy,
                    0.92,
                    scale_y,
                    [0.0, 0.0, 0.0, 1.0],
                    wrapped,
                );
            }
            text::push_font_text_xy(r, f, x, y, 0.92, scale_y, color, wrapped);
        }
        // The bitmap fallback keeps one row per '\n' at the clamped scale
        // (no kerning metrics — the 8 px stand-in, docs/ANALYSIS.md).
        None => {
            let step = 8.0 * scale_y;
            for (i, line) in wrapped.split('\n').enumerate() {
                text::hud_item_text(r, lookups, x, y + i as f32 * step, 0.92, color, line);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::achievements::{AchievementManager, Category};

    fn screen() -> Vec2 {
        Vec2::new(1600.0, 900.0)
    }

    #[test]
    fn the_panel_is_the_c_sharp_container() {
        // `Width 0.8 * 1600 = 1280` capped to MaxWidth 900, centred; the
        // game-menu arm is Top 220 with Height = screenH - 220 = 680.
        let p = panel_rect(screen(), true);
        assert_eq!(p, [350.0, 220.0, 900.0, 680.0]);
        // The in-game fancy-UI arm (`OnActivate`'s else) drops to 120/-120.
        let p = panel_rect(screen(), false);
        assert_eq!(p, [350.0, 120.0, 900.0, 780.0]);
        // A narrow screen clamps to MinWidth 700 (`600 + 100`).
        let narrow = panel_rect(Vec2::new(640.0, 480.0), true);
        assert_eq!(narrow[2], 700.0);
    }

    #[test]
    fn the_list_and_rows_follow_the_layout_constants() {
        let p = panel_rect(screen(), true);
        let inner = inner_panel(&p);
        // The UIPanel is the container minus 110.
        assert_eq!(inner[3], p[3] - 110.0);
        let list = list_rect(&p);
        // Width -25, Height -50, Top 50 against the panel's INNER rect —
        // the 12 px left/right padding insets X and width, the zeroed
        // PaddingTop keeps Y at the panel's own top (W4-g53).
        assert_eq!(
            (list[0], list[2], list[3]),
            (p[0] + 12.0, p[2] - 24.0 - 25.0, inner[3] - 12.0 - 50.0)
        );
        assert_eq!(list[1], p[1] + 50.0);
        // The first row sits at the list's top; the second is a row plus the
        // 5 px ListPadding below it.
        let r0 = row_rect(&list, 0);
        let r1 = row_rect(&list, 1);
        assert_eq!(r0[1], list[1]);
        assert_eq!(r1[1] - r0[1], ROW_H + 5.0);
        assert_eq!(r0[3], ROW_H);
    }

    #[test]
    fn the_scrollbar_rides_the_lists_right_edge() {
        // `UIScrollbar`: Width 20, Height -50, Top 50, HAlign 1
        // (`UIAchievementsMenu.cs:81-86`) against the panel-inner rect.
        let p = panel_rect(screen(), true);
        let bar = scrollbar_rect(&p);
        let list = list_rect(&p);
        assert_eq!(bar[2], 20.0);
        assert_eq!(bar[0] + bar[2], p[0] + p[2] - 12.0);
        assert_eq!((bar[1], bar[3]), (list[1], list[3]));
        // The list's -25 leaves the 20 px bar a 5 px gap.
        assert_eq!(bar[0] - (list[0] + list[2]), 5.0);
    }

    #[test]
    fn the_content_height_and_scroll_range_are_the_ui_list_walk() {
        // `UIList.RecalculateChildren` (`UIList.cs:116-128`): N rows carry
        // N paddings, the single row drops its own, none is zero.
        assert_eq!(list_content_height(0), 0.0);
        assert_eq!(list_content_height(1), ROW_H);
        assert_eq!(list_content_height(137), 137.0 * (ROW_H + 5.0));
        let p = panel_rect(screen(), true);
        let list = list_rect(&p);
        // `SetView`'s clamp bound = content - view (`UIScrollbar.cs:75`).
        let max = max_scroll(137, &list);
        assert_eq!(max, 137.0 * (ROW_H + 5.0) - list[3]);
        // A single short row never scrolls.
        assert_eq!(max_scroll(1, &list), 0.0);
        // One wheel notch is 120 px (`UIList.cs:112`, PlayerInput's raw
        // counter), clamped into the range.
        assert_eq!(scroll_wheel(500.0, 1, 137, &list), 380.0);
        assert_eq!(scroll_wheel(50.0, 1, 137, &list), 0.0);
        assert_eq!(scroll_wheel(max - 10.0, -1, 137, &list), max);
    }

    #[test]
    fn the_scrollbar_press_grabs_the_handle_or_jumps_the_track() {
        let p = panel_rect(screen(), true);
        let bar = scrollbar_rect(&p);
        let list = list_rect(&p);
        let content = list_content_height(137);
        let handle = super::super::panel::scrollbar_handle_at(bar, list[3], content, 0.0);
        // On the handle: the grab keeps the pointer offset, no jump.
        let mid = Vec2::new(handle[0] + 10.0, handle[1] + 4.0);
        assert_eq!(scrollbar_press(&p, 137, 0.0, mid), Some((0.0, Some(4.0))));
        // On the track below the handle: the jump centres the handle
        // (`UIScrollbar.cs:140`), no drag latch.
        let track = Vec2::new(bar[0] + 10.0, bar[1] + bar[3] - 8.0);
        let (pos, drag) = scrollbar_press(&p, 137, 0.0, track).unwrap();
        assert!(drag.is_none());
        assert!(pos > 0.0 && pos <= max_scroll(137, &list));
        // Off the bar entirely: not the scrollbar's press.
        assert_eq!(scrollbar_press(&p, 137, 0.0, Vec2::new(bar[0] - 5.0, bar[1])), None);
        // A single-row list has no scrollable bar.
        assert_eq!(scrollbar_press(&p, 1, 0.0, mid), None);
        // The drag frame follows the pointer with the latched offset.
        let dragged = scrollbar_drag(&p, 137, 4.0, handle[1] + 4.0 + 120.0);
        assert!(dragged > 0.0);
    }

    #[test]
    fn the_back_and_category_panels_sit_in_the_container() {
        let p = panel_rect(screen(), true);
        let back = back_rect(&p);
        // Width -10 at 0.5, Height 50, Top -45 above the container's bottom.
        assert_eq!(back[2], p[2] * 0.5 - 10.0);
        assert_eq!(back[1] + back[3], p[1] + p[3] - 45.0);
        // The four toggles: 32 px, `Left = j * 36 + 8` inside the strip,
        // which rides the panel's 12 px left padding (W4-g53).
        let cats = category_rects(&p);
        for j in 0..4 {
            assert_eq!(cats[j][2], CAT_SIZE);
            assert_eq!(cats[j][0], p[0] + 12.0 + j as f32 * 36.0 + 8.0);
            assert_eq!(cats[j][1], p[1] + 10.0);
        }
    }

    #[test]
    fn the_filter_keeps_only_the_toggled_categories() {
        let all = [true; 4];
        let none = [false; 4];
        assert!(filter_visible(&all, Category::Slayer));
        assert!(!filter_visible(&none, Category::Challenger));
        // A single filter keeps just its rows.
        let only_slayer = [true, false, false, false];
        assert!(filter_visible(&only_slayer, Category::Slayer));
        assert!(!filter_visible(&only_slayer, Category::Explorer));
        // No registry row is None (the CLR enum-field default is Slayer,
        // `Achievement.cs:25`); the defensive arm hides rather than throws
        // (W4-g53 census correction — vanilla never indexes -1).
        assert!(!filter_visible(&all, Category::None));
    }

    #[test]
    fn the_row_height_and_icon_offsets_split_on_large() {
        // `UIAchievementListItem.cs:49-53`: 66 + 16 + large*20.
        assert_eq!(row_height(true), 102.0);
        assert_eq!(row_height(false), 82.0);
        assert_eq!(row_height(LARGE), ROW_H);
        // `:50-51`, `:64-69`: icon (6, 12) large, borders 4 px out.
        assert_eq!(icon_offsets(true), ([6.0, 12.0], [2.0, 8.0]));
        assert_eq!(icon_offsets(false), ([0.0, 0.0], [-4.0, -4.0]));
    }

    #[test]
    fn the_icon_frame_walks_the_8_by_66_sheet() {
        // `:57-61`: index 9 -> col 1, row 1; locked shifts +528.
        assert_eq!(icon_frame_rect(0, false), [0.0, 0.0, 64.0, 64.0]);
        assert_eq!(icon_frame_rect(9, false), [66.0, 66.0, 64.0, 64.0]);
        assert_eq!(icon_frame_rect(9, true), [594.0, 66.0, 64.0, 64.0]);
    }

    #[test]
    fn the_description_scale_clamps_at_the_language_cap() {
        // `:101-112`: cap 58 large / 38 small, scale 0.92 * cap/height.
        assert_eq!(description_scale_y(40.0, true), 0.92);
        assert_eq!(description_scale_y(58.0, true), 0.92);
        assert_eq!(description_scale_y(116.0, true), 0.92 * 0.5);
        assert_eq!(description_scale_y(76.0, false), 0.92 * 0.5);
    }

    /// W4-g53 witness: the description wraps through d121's ReLogic port
    /// at `(num2 - 20) * (1/0.92)` (`UIAchievementListItem.cs:102`) and the
    /// Y scale clamps at the 58 px cap (`:108-112`). With no retail font
    /// the bitmap fallback measures 8 px/char, 8 px rows — "aaaa bbbb"
    /// (9 chars) at text_w 60 wraps at `(60-20)/0.92 ≈ 43.5` px: "aaaa "
    /// (5 chars = 40) fits, "bbbb" wraps; two rows measure 16 px unscaled,
    /// 14.72 at 0.92 — under the cap, so the scale stays 0.92.
    #[test]
    fn the_description_wraps_at_the_relogic_width() {
        let lookups = Lookups::default();
        let (wrapped, scale_y) = description_layout(&lookups, "aaaa bbbb", 60.0);
        assert_eq!(wrapped, "aaaa \nbbbb");
        assert_eq!(scale_y, 0.92);
        // A long description wraps to many rows: 9 rows * 8 px * 0.92 =
        // 66.24 > 58 clamps the Y scale to 0.92 * 58/66.24 (each "aaaa "
        // pair fits 40 px on a row — the space token rides the word it
        // follows, d121's `Append` `:96-100`).
        let long = "aaaa ".repeat(9); // 45 chars -> 9 "aaaa " rows
        let (wrapped, scale_y) = description_layout(&lookups, &long, 60.0);
        let rows = wrapped.split('\n').count();
        assert_eq!(rows, 9);
        let h = rows as f32 * 8.0 * 0.92;
        assert_eq!(scale_y, 0.92 * (58.0 / h));
        // A short one-liner never wraps nor clamps.
        let (wrapped, scale_y) = description_layout(&lookups, "short", 400.0);
        assert_eq!(wrapped, "short");
        assert_eq!(scale_y, 0.92);
    }

    #[test]
    fn the_tracker_bar_grows_from_the_centre_out() {
        // `DrawProgressBar` `:241-256` with Width 80 (`:138`): the anchor is
        // spot + (1, 8) + ((num - 0.5) * 80) - 1 on X.
        let spot = [100.0, 50.0];
        let (anchor, fill, rest, blip) = tracker_bar_geometry(spot, 80.0, 0.5);
        assert_eq!(anchor, [100.0, 58.0]);
        assert_eq!((fill, rest, blip), (40.0, 40.0, true));
        // Zero progress: no blip (`:252`), the anchor at the left edge.
        let (anchor, fill, rest, blip) = tracker_bar_geometry(spot, 80.0, 0.0);
        assert_eq!(anchor, [60.0, 58.0]);
        assert_eq!((fill, rest, blip), (0.0, 80.0, false));
        // The clamp (`:241`): 1.5 -> 1.
        let (anchor, fill, rest, _) = tracker_bar_geometry(spot, 80.0, 1.5);
        assert_eq!(anchor, [140.0, 58.0]);
        assert_eq!((fill, rest), (80.0, 0.0));
    }

    #[test]
    fn the_tracker_colors_dim_by_a_quarter_when_idle() {
        // `:139-148`: Lerp(c, Black, 0.25) — (100,255,100) -> (75,191,75),
        // White -> (191,191,191); the blip is the fill at alpha 127 (`:149`).
        let (track, fill, blip) = tracker_bar_colors(true);
        assert_eq!(
            (track, fill, blip),
            (
                [255, 255, 255, 255],
                [100, 255, 100, 255],
                [100, 255, 100, 127]
            )
        );
        let (track, fill, blip) = tracker_bar_colors(false);
        assert_eq!(track, [191, 191, 191, 255]);
        assert_eq!(fill, [75, 191, 75, 255]);
        assert_eq!(blip, [75, 191, 75, 127]);
    }

    #[test]
    fn the_visible_list_is_sorted_and_filtered() {
        let mut m = AchievementManager::registry();
        m.notify(crate::achievements::CondKind::NpcKilled(4)); // EYE_ON_YOU (Slayer)
        let rows = visible_rows(&m, &[true; 4]);
        assert_eq!(rows.len(), 137);
        // Completed first (`CompareTo`): the Slayer rows come later in the
        // registration order, so EYE_ON_YOU leads the list.
        assert_eq!(m.list[rows[0]].name, "EYE_ON_YOU");
        assert_eq!(m.list[rows[1]].name, "TIMBER");
        // One category only: exactly the Slayer rows — 31 registered +
        // GAIN_TORCH_GODS_FAVOR, whose unset `_category` defaults to
        // Slayer in the C# too (W4-g53 census correction).
        let slayers = visible_rows(&m, &[true, false, false, false]);
        assert_eq!(slayers.len(), 32);
        assert!(slayers
            .iter()
            .all(|&i| m.list[i].category == Category::Slayer));
    }

    #[test]
    fn the_row_hits_match_the_row_lattice() {
        let p = panel_rect(screen(), true);
        let list = list_rect(&p);
        let rows = AchievementManager::registry().list.len();
        // The second visible row's centre maps back to index 1.
        let rect = row_rect(&list, 1);
        assert_eq!(row_at(&list, 0.0, Vec2::new(rect[0] + 20.0, rect[1] + ROW_H * 0.5), rows), Some(1));
        // Outside the list horizontally misses.
        assert_eq!(row_at(&list, 0.0, Vec2::new(list[0] - 5.0, rect[1]), rows), None);
        // Scrolled by one row: the same pixel maps to the next row.
        assert_eq!(
            row_at(&list, ROW_H + 5.0, Vec2::new(rect[0] + 20.0, rect[1] + ROW_H * 0.5), rows),
            Some(2)
        );
    }

    #[test]
    fn the_scroll_range_is_the_list_overflow() {
        let p = panel_rect(screen(), true);
        let list = list_rect(&p);
        let rows = 137;
        let max = max_scroll(rows, &list);
        // `UIList`'s content height carries the trailing padding
        // (`UIList.cs:125-127`), so the bound is N*(ROW_H+5) - view.
        let expected = rows as f32 * (ROW_H + 5.0) - list[3];
        assert!((max - expected).abs() < 0.001);
        assert!(max > 0.0);
        // The last row is reachable at the maximum scroll.
        let last = row_rect(&list, rows - 1);
        assert!(last[1] - max + ROW_H <= list[1] + list[3] + 0.001);
    }

    #[test]
    fn the_icon_frame_is_the_eight_per_row_sheet_offset() {
        // Icon 0: column 0, row 0. Icon 88 (BENCHED): column 0, row 11.
        // The locked copy rides +528 px.
        assert_eq!(icon_col_row(0), (0.0, 0.0));
        assert_eq!(icon_col_row(8), (0.0, 1.0));
        assert_eq!(icon_col_row(88), (0.0, 11.0));
        assert_eq!(LOCKED_DX, 528.0);
    }

    fn icon_col_row(icon: u16) -> (f32, f32) {
        (
            icon as f32 % ICONS_PER_ROW,
            (icon as f32 / ICONS_PER_ROW).floor(),
        )
    }
}
