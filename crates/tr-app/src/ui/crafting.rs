//! The modern crafting panel: a faithful port of `NewCraftingUI`
//! (`decompiled/Terraria/GameContent/UI/NewCraftingUI.cs`) drawn as static chrome.
//!
//! P3 scope (spec §9.2): the cyan 9-slice frame, the title, the search bar, the
//! 11-tab filter strip, the separator, the (empty) recipe grid + auto-hidden
//! scrollbar, the detail area and the two toggles with their selected-crafting
//! indicator. Recipes stay empty and no filter is active, so the grid draws
//! nothing and nothing is interactive.
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
use crate::craft;
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
    /// The panel's content rect — padding 12 (`UIPanel.cs:32`), right 8 (`:324`).
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

/// The recipe grid cell the mouse is over, mapped back to the available
/// recipe list (`NewCraftingUI`'s clickable recipe slots). Cells are 40x40 on
/// a 44 px pitch from the grid's top-left.
pub(crate) fn hit_test(
    screen: Vec2,
    lookups: &assets::Lookups,
    mouse: Vec2,
    available: &[usize],
) -> Option<usize> {
    let l = layout(screen, title_width(lookups), false);
    let cols = ((l.grid[2] - 4.0) / 44.0).floor().max(1.0);
    for (i, recipe) in available.iter().enumerate() {
        let col = i as f32 % cols;
        let row = (i as f32) / cols;
        let x = l.grid[0] + col * 44.0;
        let y = l.grid[1] + row * 44.0;
        if mouse.x >= x && mouse.x <= x + 40.0 && mouse.y >= y && mouse.y <= y + 40.0 {
            return Some(*recipe);
        }
    }
    None
}

/// Draws the whole panel in the source's z-order. `mouse_pos` only feeds the hover
/// frames; `guide_mode` selects the `Main.InGuideCraftMenu` variants.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    mouse_pos: Vec2,
    available: &[usize],
    guide_mode: bool,
    difficulty_3_and_creative_unblocked: bool,
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

    // 5. The grid area (`UIDynamicItemCollection.cs:38-71`): the available
    //    recipes as 40x40 icon cells on a 44 px pitch — the same cells
    //    `hit_test` maps clicks back to. The hovered cell takes the cyan
    //    hover frame (`ItemSlot`'s hover, `ItemSlot.cs:2264+` family).
    for (i, recipe_idx) in available.iter().enumerate() {
        let col = i as f32 % ((l.grid[2] - 4.0) / 44.0).floor().max(1.0);
        let row = (i as f32) / ((l.grid[2] - 4.0) / 44.0).floor().max(1.0);
        let x = l.grid[0] + col * 44.0;
        let y = l.grid[1] + row * 44.0;
        let hovered = mouse_pos.x >= x
            && mouse_pos.x <= x + 40.0
            && mouse_pos.y >= y
            && mouse_pos.y <= y + 40.0;
        if hovered {
            r.push_ui_sprite([x, y], [40.0, 40.0], r.white_uv(), [0.35, 0.55, 0.85, 0.55]);
        }
        let recipe = &craft::RECIPES[*recipe_idx];
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
            // The icon fits inside the cell, centred (`ItemSlot.Draw`: scale
            // to the slot with the texture's own aspect kept).
            let scale = (32.0 / w).min(32.0 / h).min(1.0);
            let (dw, dh) = (w * scale, h * scale);
            r.push_ui_sprite(
                [x + (40.0 - dw) * 0.5, y + (40.0 - dh) * 0.5],
                [dw, dh],
                uv,
                [1.0; 4],
            );
        } else {
            // Items without an extracted sprite (the atlas carries only the
            // world's used tile frames): a grey placeholder box — documented
            // in docs/ANALYSIS.md.
            r.push_ui_sprite([x + 4.0, y + 4.0], [32.0, 32.0], r.white_uv(), [0.5; 4]);
        }
    }

    // 6. The scrollbar (`:376-384`): an empty list views 1 of 1, and
    //    `AutoHide` hides it when it cannot scroll (`UIScrollbar.cs:105`).
    panel::draw_scrollbar(r, lookups, l.scrollbar, 1.0, 1.0, true);

    // 7. The filter strip (`:387-392`), drawn over the panel subtree, matching
    //    `totalContainer`'s child order (`:330`, `:392`); no filter is active in P3.
    draw_filter_strip(r, lookups, &l, mouse_pos, [false; 11]);

    // 8. The detail area (`DrawChildren`, `:438-482`), drawn after the base pass so
    //    it sits on top of everything.
    draw_detail_area(r, lookups, &l, guide_mode);

    // 9. The toggles (`:484-495`).
    draw_toggles(r, lookups, &l, mouse_pos, guide_mode);
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
        // `num = 2 - CraftFromNearbyChests.ToInt() * 2 + hover` (`CraftingUI.cs:340`);
        // the setting defaults to true (`Player.cs:406`), so `2 - true*2` cancels
        // and `num` is just the hover bit (0, or 1 while hovered).
        let num = hover as usize;
        draw_centered_toggle(r, lookups, lookups.chest_craft[num], nx, ny);
    }
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
/// `UIImageFramed.cs:59`) trims them to 40x46.
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
        // (`UIImageFramed.cs:21`), unlit or hovered alike (`:63`'s DrawSelf).
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
    //    (`:100`, `UIImageButton.cs:31-32`). `SetVisibility(1f, 1f)` (`:108`) means
    //    it draws at full White hovered or not (`UIImageButton.cs:51`); the
    //    `Button_Search_Border` hover frame is drawn *over* the base image, not
    //    instead of it (`UIImageButton.cs:52-55`).
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
        // `UIPanel.cs:32` + `:324`: 490 - 12 - 8 = 470.
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
