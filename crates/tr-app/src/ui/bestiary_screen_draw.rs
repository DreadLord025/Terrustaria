//! The bestiary screen's chrome draws (W4-e23): the top strip's
//! page/range/sort/filter/search controls (`UIBestiaryTest.cs:220-300`),
//! the sorting/filtering option grids (`UIBestiarySortingOptionsGrid.cs`,
//! `UIBestiaryFilteringOptionsGrid.cs`) and the wrapped search bar
//! (`UIWrappedSearchBar.cs`). The state + geometry live in
//! [`super::bestiary_screen`]; the grid/info-page draws in
//! [`super::bestiary`].

use glam::Vec2;
use tr_render::SpriteRenderer;

use tr_core::bestiary::WorldBestiary;

use crate::assets::Lookups;
use crate::bestiary_filter as bf;
use crate::ui::{color, elements::GroupOptionButtonState, in_rect, panel, push_whole, text, uistate};

use super::bestiary as layout;
use super::bestiary_screen::{self as screen, BestiaryUi, OptionsGrid};

/// One `UIImageButton` (`UIImageButton.cs:48-56`): the base texture at
/// `Color * (hovered ? active : inactive)`, the hover frame OVER it while
/// hovered. The bestiary's buttons all `SetVisibility(1, 1)`
/// (`UIBestiaryTest.cs:224`/`:233`/`:266`/`:286`).
fn image_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: [f32; 4],
    base: Option<tr_render::SheetRect>,
    border: Option<tr_render::SheetRect>,
    hovered: bool,
) {
    if let Some(base) = base {
        push_whole(r, lookups, &base, rect, [1.0; 4]);
    }
    if hovered {
        if let Some(border) = border {
            push_whole(r, lookups, &border, rect, [1.0; 4]);
        }
    }
}

/// The top strip: Back/Forward page buttons, the range panel, the
/// sort/filter toggles with their live labels, the wrapped search bar.
pub fn draw_top_strip(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    scr: Vec2,
    ui: &BestiaryUi,
    mouse: Vec2,
) {
    // `AddBackAndForwardButtons` (`UIBestiaryTest.cs:220-255`).
    let (back, forward) = screen::page_buttons_rects(scr);
    image_button(r, lookups, back, lookups.ui_bestiary_page_back, lookups.ui_bestiary_page_border, in_rect(mouse, back));
    image_button(r, lookups, forward, lookups.ui_bestiary_page_forward, lookups.ui_bestiary_page_border, in_rect(mouse, forward));
    // The range panel: `(35, 40, 83)` bg AND border (`:244-245`), padding 0;
    // the range text centres at scale 0.8 (`:248-252`).
    let range_panel = screen::range_panel_rect(scr);
    let dark = color::tint([35, 40, 83, 255]);
    r.push_ui_sprite([range_panel[0], range_panel[1]], [range_panel[2], range_panel[3]], r.white_uv(), dark);
    let range = ui.range_text(scr);
    let rw = text::hud_text_width(r, lookups, &range, 0.8);
    let rh = text::line_height(lookups) * 0.8;
    text::hud_text(
        r,
        lookups,
        range_panel[0] + (range_panel[2] - rw) * 0.5,
        range_panel[1] + (range_panel[3] - rh) * 0.5,
        0.8,
        [1.0; 4],
        &range,
    );

    // `AddSortAndFilterButtons` (`:257-300`): the two 114x28 toggles with
    // the wide hover border and the live label texts (`_sortingText` =
    // `sorter.GetDisplayName()`, `_filteringText` = `filterer.GetDisplayName()`,
    // `UpdateBestiaryContents:435-436`).
    let (sort_btn, filter_btn) = screen::sort_filter_buttons_rects(scr);
    image_button(r, lookups, sort_btn, lookups.ui_bestiary_sort_button, lookups.ui_bestiary_wide_border, in_rect(mouse, sort_btn));
    image_button(r, lookups, filter_btn, lookups.ui_bestiary_filter_button, lookups.ui_bestiary_wide_border, in_rect(mouse, filter_btn));
    let lh = text::line_height(lookups) * 0.8;
    let sort_label = ui.sorter.display_name();
    let p = screen::button_label_pos(sort_btn, lh);
    text::hud_text(r, lookups, p[0], p[1], 0.8, [1.0; 4], &sort_label);
    let filter_label = ui.filterer.display_name();
    let p = screen::button_label_pos(filter_btn, lh);
    text::hud_text(r, lookups, p[0], p[1], 0.8, [1.0; 4], &filter_label);

    draw_search_bar(r, lookups, scr, ui, mouse);
}

/// The wrapped search bar (`UIWrappedSearchBar.cs:95-150`), the Blue theme:
/// the `Button_Search` magnifier (full White either way, `:108`), the
/// `(35, 40, 83)` box panel (the focus border swaps to OurFavoriteColor,
/// `:199-211`), the placeholder/contents text and the 24x24 cancel button
/// (0.4 dim at rest, `UIImageButton.cs:15`).
fn draw_search_bar(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    scr: Vec2,
    ui: &BestiaryUi,
    mouse: Vec2,
) {
    let bar = screen::search_bar_rect(scr);
    let (button, box_rect, cancel) = screen::search_pieces(bar);
    image_button(r, lookups, button, lookups.ui_search_icon_blue, lookups.ui_search_border, in_rect(mouse, button));
    // The box: bg + border panels, the border takes the live focus colour.
    let bg = color::tint([35, 40, 83, 255]);
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, box_rect, bg);
    panel::draw_panel(r, lookups, lookups.ui_panel_border, box_rect, color::tint(ui.search.border_color));
    // The inner bar's text (`UISearchBar.DrawSelf` → the UITextBox):
    // Gray placeholder / White contents (`SearchBar::text_color`), the
    // box's 12 px left padding + the 4 px vertical pin (the crafting bar's
    // `search_text_pos` form, `UITextPanel.cs:92-109`).
    let tc = color::tint(ui.search.bar.text_color());
    text::hud_text(r, lookups, box_rect[0] + 12.0, box_rect[1] + 4.0, 0.8, tc, &ui.search.bar.display_text);
    // The cancel button: never SetVisibility — 0.4 at rest, White hovered.
    let cancel_tint = if in_rect(mouse, cancel) {
        [1.0; 4]
    } else {
        color::tint(color::mul([255, 255, 255, 255], 0.4))
    };
    if let Some(sheet) = lookups.ui_search_cancel {
        push_whole(r, lookups, &sheet, cancel, cancel_tint);
    }
}

/// The attached option grid over the bestiary space.
pub fn draw_option_grid(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    scr: Vec2,
    tracker: &WorldBestiary,
    ui: &BestiaryUi,
    mouse: Vec2,
) {
    match ui.options {
        OptionsGrid::None => {}
        OptionsGrid::Sorting => draw_sorting_grid(r, lookups, scr, ui, mouse),
        OptionsGrid::Filtering => draw_filtering_grid(r, lookups, scr, tracker, ui, mouse),
    }
}

/// The option grids' shared panel chrome: the full-space `(35, 40, 83) *
/// 0.5` backdrop (both ctors, `UIBestiarySortingOptionsGrid.cs:30-31` /
/// `UIBestiaryFilteringOptionsGrid.cs:36-37`) + the container's
/// `(89, 116, 213) * 0.9` border / `(73, 94, 171) * 0.9` bg pair
/// (`:58-59` / `:55-56`).
fn option_panel_chrome(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    scr: Vec2,
    container: [f32; 4],
) {
    let space = layout::bestiary_space(&layout::panel_rect(scr));
    let half = color::tint(color::mul([35, 40, 83, 255], 0.5));
    r.push_ui_sprite([space[0], space[1]], [space[2], space[3]], r.white_uv(), half);
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, container, color::tint(color::mul([73, 94, 171, 255], 0.9)));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, container, color::tint(color::mul([89, 116, 213, 255], 0.9)));
}

/// One `GroupOptionButton` fill/border/icon stack
/// (`GroupOptionButton.cs:233-249`) over the shared colour state — the
/// bestiary grids keep `ShowHighlightWhenSelected = false`
/// (`UIBestiarySortingOptionsGrid.cs:75`, `UIBestiaryFilteringOptionsGrid.cs:106`),
/// so the selected rim never draws; the hover border does.
fn option_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: [f32; 4],
    state: &GroupOptionButtonState,
    selected: bool,
    hovered: bool,
) {
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_gray,
        rect,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(state.fill_color(selected)),
    );
    if hovered {
        uistate::draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_border,
            rect,
            10.0,
            10.0,
            10.0,
            10.0,
            color::tint(state.hover_border_color()),
        );
    }
}

/// `UIBestiarySortingOptionsGrid` (`:37-115`): the 126-wide container at
/// `Left -118`, one 114x26 button per visible step at `Top 5 + 28n`, the
/// step's display name centred at scale 0.8, the selection colour pair
/// `(152, 175, 235)` @ 1 / InventoryDefaultColor @ 0.7.
fn draw_sorting_grid(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    scr: Vec2,
    ui: &BestiaryUi,
    mouse: Vec2,
) {
    let container = screen::sort_grid_rect(scr);
    option_panel_chrome(r, lookups, scr, container);
    for (n, step) in screen::visible_sort_steps().into_iter().enumerate() {
        let rect = layout::sort_button_rect(container, n);
        // `ClickOption`'s SetCurrentOption walk (`:96-108`): the button
        // whose OptionValue == _currentSelected is selected.
        let selected = ui.sort_selected == step as i32;
        let (c, opacity) = layout::sort_button_colors(selected);
        let mut state = GroupOptionButtonState::default();
        state.set_color(c, opacity);
        let hovered = in_rect(mouse, rect);
        option_button(r, lookups, rect, &state, selected, hovered);
        // The title UIText (`:68`): scale 0.8, HAlign 0.5 (the ctor
        // default), VAlign 0.5, Width (-10, 1) — centred in the button.
        let label = crate::lang::text(bf::step_display_name_key(screen::step_of(step)));
        let lw = text::hud_text_width(r, lookups, &label, 0.8);
        let lh = text::line_height(lookups) * 0.8;
        text::hud_text(
            r,
            lookups,
            rect[0] + (rect[2] - lw) * 0.5,
            rect[1] + (rect[3] - lh) * 0.5,
            0.8,
            [1.0; 4],
            &label,
        );
    }
}

/// `UIBestiaryFilteringOptionsGrid` (`:43-207`): the 12-per-row 32x32
/// button lattice; available filters carry their icon (the tag frame or
/// the rank star), unavailable ones the `-2` shadow button at
/// `FadeFromBlack 0.5` with the `(0, 4)` tag frame at White * 0.2. The
/// hovered button prints its display name (`ShowButtonName:217-224`).
fn draw_filtering_grid(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    scr: Vec2,
    tracker: &WorldBestiary,
    ui: &BestiaryUi,
    mouse: Vec2,
) {
    let container = screen::filter_grid_rect(scr);
    option_panel_chrome(r, lookups, scr, container);
    let mut hover_label: Option<String> = None;
    for i in 0..bf::FILTER_COUNT {
        let rect = screen::filter_button_rect(scr, i);
        let hovered = in_rect(mouse, rect);
        let f = bf::filter_at(i);
        let available = bf::filter_available(i, tracker);
        if available {
            let active = ui.filterer.is_active(i);
            let (c, opacity, _) = layout::filter_button_selection(active, i as i32);
            let mut state = GroupOptionButtonState::default();
            state.set_color(c, opacity);
            option_button(r, lookups, rect, &state, active, hovered);
            // The icon (`GetImage`): centred in the button (HAlign/VAlign
            // 0.5) at (-1, -1) (`UpdateAvailability:112-113`), tinted by
            // the button's colour chain (`GroupOptionButton.cs:242-249`).
            let tint = color::tint(state.icon_tint(active, hovered, [255, 255, 255, 255]));
            match bf::filter_icon_frame(f) {
                Some(frame) => {
                    if let Some(sheet) = lookups.ui_bestiary_tag_icons {
                        let dst = tag_icon_dst(rect, [sheet.width as f32 / 16.0, sheet.height as f32 / 5.0]);
                        draw_tag_frame(r, lookups, &sheet, frame, dst, tint);
                    }
                }
                None => {
                    // ByRareCreature's whole Icon_Rank_Light (`Filters.cs:107-115`).
                    if let Some(sheet) = lookups.ui_bestiary_star_light {
                        let dst = tag_icon_dst(rect, [sheet.width as f32, sheet.height as f32]);
                        push_whole(r, lookups, &sheet, dst, tint);
                    }
                }
            }
            if hovered {
                hover_label = Some(crate::lang::text(bf::filter_display_name_key(f)));
            }
        } else {
            // The unavailable button (`:118-146`): option -2 (never
            // selected), FadeFromBlack 0.5, the shadow frame (0, 4) at
            // White * 0.2 — the icon is a UIImageFramed CHILD, so the
            // button's colour chain does not tint it.
            let (option, fade, icon_tint) = layout::unavailable_filter_button();
            let _ = option;
            let mut state = GroupOptionButtonState::default();
            state.fade_from_black = fade;
            option_button(r, lookups, rect, &state, false, hovered);
            if let Some(sheet) = lookups.ui_bestiary_tag_icons {
                let dst = tag_icon_dst(rect, [sheet.width as f32 / 16.0, sheet.height as f32 / 5.0]);
                draw_tag_frame(r, lookups, &sheet, 4 * 16, dst, color::tint(icon_tint));
            }
        }
    }
    if let Some(label) = hover_label {
        super::tooltip::draw_mouse_text(r, lookups, mouse, &label, scr, super::UiPulse::default());
    }
}

/// The filter icon's destination: the frame centred in the button
/// (`HAlign/VAlign 0.5`) shifted (-1, -1) (`UpdateAvailability:112-113`).
fn tag_icon_dst(button: [f32; 4], size: [f32; 2]) -> [f32; 4] {
    [
        button[0] + (button[2] - size[0]) * 0.5 - 1.0,
        button[1] + (button[3] - size[1]) * 0.5 - 1.0,
        size[0],
        size[1],
    ]
}

/// One `Icon_Tags_Shadow` frame — the 16x5 sheet sliced by the linear
/// frame index (`FilterProviderInfoElement.cs:27-28`: `X = frame % 16`,
/// `Y = frame / 16`).
fn draw_tag_frame(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    sheet: &tr_render::SheetRect,
    frame: u32,
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let fw = sheet.width / 16;
    let fh = sheet.height / 5;
    let x0 = sheet.x + (frame % 16) * fw;
    let y0 = sheet.y + (frame / 16) * fh;
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let uv = [
        x0 as f32 / aw,
        y0 as f32 / ah,
        (x0 + fw) as f32 / aw,
        (y0 + fh) as f32 / ah,
    ];
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_tag_icon_centres_with_the_minus_one_shift() {
        // A 30x30 frame in a 32x32 button at (100, 50): centre (101, 51)
        // minus (1, 1) -> (100, 50)... the frame's top-left = button + 1 - 1.
        let dst = tag_icon_dst([100.0, 50.0, 32.0, 32.0], [30.0, 30.0]);
        assert_eq!(dst, [100.0, 50.0, 30.0, 30.0]);
        // A 14x14 star: (32-14)/2 - 1 = 8.
        let dst = tag_icon_dst([100.0, 50.0, 32.0, 32.0], [14.0, 14.0]);
        assert_eq!(dst, [108.0, 58.0, 14.0, 14.0]);
    }

    #[test]
    fn the_unavailable_shadow_frame_is_row_four_col_zero() {
        // `asset.Frame(16, 5, 0, 4)` (`UIBestiaryFilteringOptionsGrid.cs:135`)
        // — linear frame 64 on the 480x150 sheet: (0, 120) 30x30.
        let sheet = tr_render::SheetRect { x: 0, y: 0, width: 480, height: 150, plain_frame: (0, 0) };
        let frame = 4 * 16u32;
        let fw = sheet.width / 16;
        let fh = sheet.height / 5;
        assert_eq!((fw, fh), (30, 30));
        assert_eq!((frame % 16) * fw, 0);
        assert_eq!((frame / 16) * fh, 120);
        // The unlock filter's (14, 3) frame: linear 62 -> (420, 90).
        let f = bf::filter_icon_frame(bf::Filter::UnlockState).unwrap();
        assert_eq!(((f % 16) * fw, (f / 16) * fh), (420, 90));
    }
}
