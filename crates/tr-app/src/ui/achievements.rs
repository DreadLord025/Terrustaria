//! The achievements menu (`UIAchievementsMenu` + `UIAchievementListItem`,
//! `GameContent/UI/States/UIAchievementsMenu.cs` and
//! `GameContent/UI/Elements/UIAchievementListItem.cs`) — the scrolling list of
//! the 137 registry rows, the four category filter toggles, the title and Back
//! panels, and the per-row icon / name / description / progress-bar layout.
//!
//! Geometry follows the source: the outer container's `Width 0.8`,
//! `MaxWidth 900`/`MinWidth 700`, `Top 220` in the game menu (`OnActivate`'s
//! `Main.gameMenu` arm lowers it to 120 in-game), the `UIPanel`'s `-110`
//! height, the list's `-25/-50` insets with `ListPadding 5`, the `66 + 16 +
//! 20 = 102` row height of `largeForOtherLanguages = true` and the
//! 8-per-row 64 px icons at a 66 px stride (the locked copy rides +528 px).
//!
//! The port adds the same overlay treatment the bestiary uses: the state
//! draws over the menu/world backdrop, dimmed, with its own scroll and
//! category-filter state.

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text;
use super::uistate::{self, Style};
use crate::achievements::{Achievement, AchievementManager, Category};
use crate::assets::{self, Lookups};
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

/// The item panel's colours (`UIAchievementListItem`'s constructor):
/// `BackgroundColor = new Color(26, 40, 89) * 0.8f`, hover
/// `new Color(46, 60, 119)`.
const ROW_BG: [f32; 4] = [26.0 / 255.0 * 0.8, 40.0 / 255.0 * 0.8, 89.0 / 255.0 * 0.8, 0.8];
const ROW_BG_HOVER: [f32; 4] = [46.0 / 255.0, 60.0 / 255.0, 119.0 / 255.0, 1.0];

/// `UIAchievementsMenu`'s `UIPanel` colour: `new Color(33, 43, 79) * 0.8f`
/// (`:…`), the same tone the bestiary's panel draws.
const PANEL_BG: [f32; 4] = [33.0 / 255.0 * 0.8, 43.0 / 255.0 * 0.8, 79.0 / 255.0 * 0.8, 0.8];

/// `new Color(73, 94, 171)` — the title panel's `BackgroundColor`.
const TITLE_BG: [f32; 4] = [73.0 / 255.0, 94.0 / 255.0, 171.0 / 255.0, 1.0];

/// The `UIPanel` default border (the port's panel helper draws it with
/// `ui_panel_border`); `UITextPanel`'s is `Color.Black`.
const TITLE_BORDER: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

/// The unlocked/locked name tints — `Color.Lerp(Silver/Gold, White, hover)`.
const SILVER: [f32; 4] = [0.753, 0.753, 0.753, 1.0];
const GOLD: [f32; 4] = [1.0, 0.843, 0.0, 1.0];
const DESCRIPTION_LOCKED: [f32; 4] = [0.663, 0.663, 0.663, 1.0]; // `Color.DarkGray`
const DESCRIPTION_OPEN: [f32; 4] = [0.753, 0.753, 0.753, 1.0]; // `Color.Silver`
const CATEGORY_ICON_TINT: [f32; 4] = [0.753, 0.753, 0.753, 1.0];

/// The progress bar's colours (`DrawProgressBar`'s `color2`/`color3`):
/// `new Color(100, 255, 100)` fill, `new Color(255, 255, 255)` track — the
/// hover arm lerps them 25 % toward black.
const BAR_FILL: [f32; 4] = [100.0 / 255.0, 1.0, 100.0 / 255.0, 1.0];
const BAR_TRACK: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

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
/// (`:…`), which the category strip and the scrollbar hang off.
pub fn inner_panel(panel: &[f32; 4]) -> [f32; 4] {
    [panel[0], panel[1], panel[2], panel[3] - 110.0]
}

/// The `UIList` — `Width -25`, `Height -50`, `Top 50`, `ListPadding 5`.
pub fn list_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = inner_panel(panel);
    [
        inner[0],
        inner[1] + 50.0,
        inner[2] - 25.0,
        (inner[3] - 50.0).max(0.0),
    ]
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

/// The four category toggles — the 32 px strip `Top 10` inside the
/// `UIPanel`, with `Left = j * 36 + 8`.
pub fn category_rects(panel: &[f32; 4]) -> [[f32; 4]; 4] {
    let inner = inner_panel(panel);
    std::array::from_fn(|j| {
        [
            inner[0] + (j as f32 * 36.0 + 8.0),
            inner[1] + 10.0,
            CAT_SIZE,
            CAT_SIZE,
        ]
    })
}

/// Rows per viewport — the list's height over the row stride.
pub fn rows_on_screen(list: &[f32; 4]) -> usize {
    ((list[3] + 5.0) / (ROW_H + 5.0)).floor().max(1.0) as usize
}

/// The scroll range: total list height minus the viewport.
pub fn max_scroll(rows: usize, list: &[f32; 4]) -> f32 {
    (rows as f32 * (ROW_H + 5.0) - list[3] - 5.0).max(0.0)
}

/// `FilterList` (`:…`): a row shows when its category's toggle is on. The
/// secret `None` row has no toggle — vanilla's `_categoryButtons[-1]` would
/// throw, so the port keeps it visible in every view (documented deviation).
pub fn filter_visible(categories: &[bool; 4], category: Category) -> bool {
    match category.index() {
        -1 => true,
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
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    manager: &AchievementManager,
    scroll: f32,
    categories: &[bool; 4],
    mouse: Vec2,
    game_menu: bool,
) {
    let panel = panel_rect(screen, game_menu);
    r.push_ui_sprite([0.0, 0.0], [screen.x, screen.y], r.white_uv(), BACKDROP);

    // The `UIPanel` body (`new Color(33, 43, 79) * 0.8f`).
    let inner = inner_panel(&panel);
    r.push_ui_sprite(
        [inner[0], inner[1]],
        [inner[2], inner[3]],
        r.white_uv(),
        PANEL_BG,
    );

    draw_title(r, lookups, &panel);
    draw_categories(r, lookups, &panel, categories, mouse);
    draw_back(r, lookups, &panel, mouse);
    draw_list(r, lookups, &panel, manager, scroll, categories, mouse);
}

/// The `UITextPanel` title — `UI.Achievements` at scale 1.0 through the
/// large (Death) font, the panel's `(73, 94, 171)` background and black
/// border, `SetPadding(13)`.
fn draw_title(r: &mut SpriteRenderer, lookups: &Lookups, panel: &[f32; 4]) {
    let label = lang::text(TITLE_KEY);
    let rect = title_rect(panel, label_width(lookups, &label));
    // The port's panel helper splices `UI/PanelBackground`; the C# state's
    // own `UITextPanel` colour pair is drawn as the flat panel beneath it.
    r.push_ui_sprite(
        [rect[0], rect[1]],
        [rect[2], rect[3]],
        r.white_uv(),
        TITLE_BG,
    );
    super::uistate::draw_text_panel(
        r,
        lookups,
        rect,
        &title_style(),
        &label,
        label_width(lookups, &label),
        false,
    );
    let _ = TITLE_BORDER;
}

/// The large-title style — `UITextPanel`'s `large: true` scale-1 form.
fn title_style() -> uistate::TextPanelStyle {
    uistate::TextPanelStyle {
        text_scale: 1.0,
        large: true,
        padding: 13.0,
        bg: [0, 0, 0, 0],
        border: [0, 0, 0, 0],
    }
}

/// `UITextPanel.MeasureString` through the Death font (`UITextPanel.cs:126-131`).
fn label_width(lookups: &Lookups, label: &str) -> f32 {
    match &lookups.death_font {
        Some(f) => text::font_width(f, label, 1.0),
        None => 100.0,
    }
}

/// The four `UIToggleImage` filter buttons with their tooltips
/// (`Draw`'s category switch, `:…`).
fn draw_categories(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    panel: &[f32; 4],
    categories: &[bool; 4],
    mouse: Vec2,
) {
    for (j, rect) in category_rects(panel).iter().enumerate() {
        let hovered = in_rect(*rect, mouse);
        // `UIToggleImage`'s off/on frames: `Point(34 * j, 0)` / `Point(34 * j, 34)`.
        let frame_y = if categories[j] { CAT_CELL } else { 0.0 };
        draw_category_frame(
            r,
            lookups,
            j,
            frame_y,
            [rect[0], rect[1], CAT_SIZE, CAT_SIZE],
            if hovered { [1.0; 4] } else { [0.75, 0.75, 0.75, 1.0] },
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

/// The Back panel — `UITextPanel("UI.Back", 0.7f, large: true)` with the
/// `FadedMouseOver`/`FadedMouseOut` colour pair.
fn draw_back(r: &mut SpriteRenderer, lookups: &Lookups, panel: &[f32; 4], mouse: Vec2) {
    let rect = back_rect(panel);
    let label = lang::text(BACK_KEY);
    let style = super::cards::fat_button_style();
    let text_w = text::hud_text_width(r, lookups, &label, style.text_scale);
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

/// The filtered, sorted list (`FilterList` + `UpdateOrder`).
fn draw_list(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    panel: &[f32; 4],
    manager: &AchievementManager,
    scroll: f32,
    categories: &[bool; 4],
    mouse: Vec2,
) {
    let list = list_rect(panel);
    let rows = visible_rows(manager, categories);
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
        draw_row(r, lookups, &shifted, a, in_rect(shifted, mouse));
    }
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
    // The item panel (`BackgroundColor`/hover pair).
    r.push_ui_sprite(
        [rect[0], rect[1]],
        [rect[2], rect[3]],
        r.white_uv(),
        if hover { ROW_BG_HOVER } else { ROW_BG },
    );

    // `PaddingTop 8` / `PaddingLeft 9`.
    let ix = rect[0] + 9.0 + LARGE_LEFT;
    let iy = rect[1] + 8.0 + LARGE_TOP;

    // The icon — locked frame until the row completes.
    draw_icon(r, lookups, ix, iy, a.icon, locked);

    // The border, 4 px out on both axes.
    if let Some(sheet) = lookups.ach_borders {
        super::push_whole(
            r,
            lookups,
            &sheet,
            [ix - 4.0, iy - 4.0, sheet.width as f32, sheet.height as f32],
            if hover { [1.0; 4] } else { [0.75, 0.75, 0.75, 1.0] },
        );
    }

    let border_w = lookups.ach_borders.map_or(72.0, |s| s.width as f32);
    // `vector2 = border dimensions' right edge + 7` and
    // `num2 = inner width - border width + 1 - (large ? 12 : 0)`.
    let text_x = ix + border_w + 7.0 + LARGE_LEFT;
    let text_w = (rect[2] - 18.0 - LARGE_LEFT) - border_w + 1.0 - 12.0;

    // The name row — `Color.Lerp(Silver/Gold, White, hover 0.5)`.
    let name_y = iy - 2.0;
    draw_inner_panel_top(r, lookups, text_x, name_y, text_w);
    // The category icon at 0.5 scale, `Frame(4, 2, category)`.
    let cat_x = text_x + 4.0;
    let cat_y = name_y + 2.0;
    draw_category_frame(
        r,
        lookups,
        a.category.index().max(0) as usize,
        0.0,
        [cat_x, cat_y, CAT_SIZE, CAT_SIZE],
        if hover { [1.0; 4] } else { CATEGORY_ICON_TINT },
    );
    // `vector4.X += 4; vector4.X += 17`
    let name_x = cat_x + 17.0;
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

    // The description row — 27 px under the name row.
    let desc_y = iy + 27.0;
    draw_inner_panel_bottom(r, lookups, text_x + 8.0, desc_y + 4.0, text_w);
    let desc_color = lerp(if locked { DESCRIPTION_LOCKED } else { DESCRIPTION_OPEN }, [1.0; 4], if hover { 1.0 } else { 0.0 });
    let desc = lang::text(&desc_key(a.name));
    push_clipped_text(
        r,
        lookups,
        text_x + 8.0,
        desc_y + 4.0,
        0.92,
        desc_color,
        &desc,
        text_w - 8.0,
    );

    // The tracker bar — only while locked and only with a tracker
    // (`flag = (Item1 != 0 || Item2 != 0) && _locked`).
    if let Some((value, max)) = a.tracker_values() {
        if locked && (value != 0 || max != 0) {
            draw_progress(r, lookups, name_x, name_y, text_w, value, max, hover);
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

/// `DrawPanelTop` — three 2 px-wide splices of
/// `Achievement_InnerPanelTop`, the middle stretched to `width - 4`.
fn draw_inner_panel_top(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    x: f32,
    y: f32,
    width: f32,
) {
    let Some(sheet) = lookups.ach_inner_top else {
        return;
    };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let h = sheet.height as f32;
    let base_y = sheet.y as f32;
    // The left cap.
    splice(r, lookups, sheet.x as f32, base_y, 2.0, h, x, y, 2.0, h, aw, ah);
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
    );
}

/// `DrawPanelBottom` — the 6/7/6 px splices of
/// `Achievement_InnerPanelBottom` (the `_Large` variant while large).
fn draw_inner_panel_bottom(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    x: f32,
    y: f32,
    width: f32,
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
) {
    let uv = [sx / aw, sy / ah, (sx + sw) / aw, (sy + sh) / ah];
    r.push_ui_sprite([dx, dy], [dw, dh], uv, [1.0; 4]);
}

/// `DrawProgressBar` with `Width = 80`: the `ColorBar` track, the
/// `MagicPixel` fill and the bar, then the `value / max` readout at scale
/// 0.75 (`num6 = new Vector2(0.75f)`).
fn draw_progress(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    name_x: f32,
    name_y: f32,
    text_w: f32,
    value: i32,
    max: i32,
    hover: bool,
) {
    let progress = if max > 0 {
        (value as f32 / max as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };
    // `vector5 = name position + UnitX*num2 + UnitY`.
    let mut x = name_x + text_w + 1.0;
    let y = name_y + 1.0;
    let readout = format!("{value}/{max}");
    let readout_w = text::hud_item_text_width(r, lookups, &readout, 0.75);
    // The bar rides `vector5 - UnitX * num4 * 0.7f`.
    let bar_x = x - BAR_W * 0.7;
    let fill = if hover { BAR_FILL } else { lerp(BAR_FILL, [0.0; 4], 0.25) };
    let track = if hover { BAR_TRACK } else { lerp(BAR_TRACK, [0.0; 4], 0.25) };
    r.push_ui_sprite([bar_x, y], [BAR_W, BAR_H], r.white_uv(), track);
    r.push_ui_sprite(
        [bar_x, y],
        [BAR_W * progress, BAR_H],
        r.white_uv(),
        fill,
    );
    // The readout at `vector5.X -= num4 * 1.4f + stringSize2.X`.
    x -= BAR_W * 1.4 + readout_w;
    text::hud_item_text(r, lookups, x, y, 0.75, [1.0; 4], &readout);
}

/// `FontAssets.ItemStack.Value.CreateWrappedText`'s clamp — the port has no
/// word-wrap yet, so the row's text is drawn at the source scale and clipped
/// to the row's text width by the caller (documented deviation: the C#
/// wraps to a second line, growing the row while the text stays short).
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
        // Shrink to fit rather than overflowing the row (`vector3.Y *=
        // num3 / stringSize.Y` is the C#'s vertical clamp; the port's is
        // the horizontal twin).
        let scale = (scale * max_w / w).max(0.4);
        text::hud_item_text(r, lookups, x, y, scale, color, text);
        return;
    }
    text::hud_item_text(r, lookups, x, y, scale, color, text);
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
        // Width -25, Height -50, Top 50 inside the panel.
        assert_eq!((list[0], list[2], list[3]), (p[0], p[2] - 25.0, inner[3] - 50.0));
        // The first row sits at the list's top; the second is a row plus the
        // 5 px ListPadding below it.
        let r0 = row_rect(&list, 0);
        let r1 = row_rect(&list, 1);
        assert_eq!(r0[1], list[1]);
        assert_eq!(r1[1] - r0[1], ROW_H + 5.0);
        assert_eq!(r0[3], ROW_H);
    }

    #[test]
    fn the_back_and_category_panels_sit_in_the_container() {
        let p = panel_rect(screen(), true);
        let back = back_rect(&p);
        // Width -10 at 0.5, Height 50, Top -45 above the container's bottom.
        assert_eq!(back[2], p[2] * 0.5 - 10.0);
        assert_eq!(back[1] + back[3], p[1] + p[3] - 45.0);
        // The four toggles: 32 px, `Left = j * 36 + 8`, Top 10, in a row.
        let cats = category_rects(&p);
        for j in 0..4 {
            assert_eq!(cats[j][2], CAT_SIZE);
            assert_eq!(cats[j][0], p[0] + j as f32 * 36.0 + 8.0);
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
        // The secret row has no toggle (`AchievementCategory.None`) and the
        // port keeps it visible rather than indexing -1.
        assert!(filter_visible(&none, Category::None));
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
        // One category only: exactly the Slayer rows, and the None row is
        // kept (deviation, documented).
        let slayers = visible_rows(&m, &[true, false, false, false]);
        assert_eq!(slayers.len(), 32);
        assert!(slayers
            .iter()
            .all(|&i| m.list[i].category == Category::Slayer
                || m.list[i].category == Category::None));
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
        let expected = rows as f32 * (ROW_H + 5.0) - list[3] - 5.0;
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
