//! The bestiary screen (`UIBestiaryTest.cs`, opened through
//! `Main.DrawBestiaryIcon`'s inventory button) — the entry grid
//! (`UIBestiaryEntryGrid`/`UIBestiaryEntryButton`), the per-entry info page
//! (`UIBestiaryEntryInfoPage`'s stat rows) and the unlock-tier gating
//! (`CommonEnemyUICollectionInfoProvider` via `tr_core::bestiary`).
//!
//! The geometry constants follow the source: the 72 px cells
//! (`UIBestiaryEntryButton.cs:27-28`), the 72 px grid division
//! (`UIBestiaryEntryGrid.cs:95-100`), the inventory button's
//! `(498, 278) 30x30` hover rect (`Main.cs:42442`).

use glam::Vec2;
use tr_render::SpriteRenderer;

use tr_core::bestiary::{UnlockState, WorldBestiary};

use super::push_whole;
use super::text;
use super::uistate::{self, Style};
use crate::assets::Lookups;
use crate::npc::{NpcKind, SlimeVariant};

/// `UIBestiaryEntryButton`'s 72 px cell (`:27-28`).
pub const CELL: f32 = 72.0;

/// The inventory's emote button — `DrawEmoteBubblesButton`'s rect
/// (`Main.cs:42369-42408`: num=534, num2=244+30+4=278, the 30x30 box).
pub const EMOTE_BUTTON_RECT: [f32; 4] = [534.0, 278.0, 30.0, 30.0];

/// `true` while the cursor is over the emote button (`:42397`'s inclusive
/// `rectangle.Contains`).
pub fn emote_button_hit(mouse: Vec2) -> bool {
    mouse.x >= EMOTE_BUTTON_RECT[0]
        && mouse.x <= EMOTE_BUTTON_RECT[0] + EMOTE_BUTTON_RECT[2]
        && mouse.y >= EMOTE_BUTTON_RECT[1]
        && mouse.y <= EMOTE_BUTTON_RECT[1] + EMOTE_BUTTON_RECT[3]
}

/// The inventory button's hover rect — `DrawBestiaryIcon`'s
/// `rectangle = new Rectangle(num, num2, 30, 30)` with the sheet-position
/// constants resolved (`Main.cs:42438-42442`: num=498, num2=244+30+4=278).
pub const BUTTON_RECT: [f32; 4] = [498.0, 278.0, 30.0, 30.0];

/// The dark backdrop behind the panel (`UIBestiaryTest`'s state draw scales
/// the game down behind `Main.DrawBackground`; the port dims instead — the
/// shared overlay treatment of the other full-screen UI).
const BACKDROP: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

/// `BuildPage`'s outer container (`UIBestiaryTest.cs:71-79`), resolved through
/// the layout pass (`UIElement.cs:306-322`) instead of by hand:
///
/// ```text
/// Width  (0, 0.875)   MaxWidth 900   MinWidth 700   (`:71-73`)
/// Top    180          Height (-220, 1)              HAlign 0.5   (`:74-77`)
/// ```
///
/// `800 + Utils.ToInt(true) * 100` is the C#'s 900/700 pair (`:69`), the
/// non-zero/zero-style modes' widths. The container's **height** is the source's
/// `screenH − 220` — the previous hand-written rect subtracted the 180 top a
/// second time (`screenH − 400`, 180 px short at the bottom); the layout pass is
/// the faithful form, so the panel now ends at `screenH − 40`.
pub fn panel_rect(screen: Vec2) -> [f32; 4] {
    uistate::resolve(
        [0.0, 0.0, screen.x, screen.y], // the `UIState` root (`UIState.cs:9-11`)
        Style::px(0.0),                 // `Left` unset (`StyleDimension.Empty`)
        Style::px(180.0),               // `Top.Set(180f, 0f)` (`:74`)
        Style::new(0.0, 0.875),         // `Width.Set(0f, 0.875f)` (`:71`)
        Style::new(-220.0, 1.0),        // `Height.Set(-220f, 1f)` (`:76`)
        0.5,                            // `HAlign = 0.5f` (`:77`)
        0.0,                            // `VAlign` unset
        700.0,                          // `MinWidth` (`:73`)
        0.0,                            // `MinHeight` unset
        900.0,                          // `MaxWidth` (`:72`)
        [0.0; 4],                       // the container carries no padding
    )
}

/// The `Back` panel — `MakeExitButton` (`:364-380`): 50 px tall, bottom
/// -25, centred, half the container's width (-10).
pub fn back_rect(panel: &[f32; 4]) -> [f32; 4] {
    let w = (panel[2] - 10.0) * 0.5;
    [
        panel[0] + panel[2] * 0.5 - w * 0.5,
        panel[1] + panel[3] - 25.0 - 50.0,
        w,
        50.0,
    ]
}

/// The main `UIPanel` — `BuildPage` (`UIBestiaryTest.cs:81-87`):
/// `Width (0, 1)`, `Height (-90, 1)` inside the outer container, the
/// `(33, 43, 79) * 0.8` background. The port draws it as a flat quad (the
/// full-screen states' panel style).
pub fn inner_panel(panel: &[f32; 4]) -> [f32; 4] {
    [panel[0], panel[1], panel[2], panel[3] - 90.0]
}

/// The panel's inner rect — `UIPanel`'s default 12 px padding with
/// `PaddingTop/Bottom -= 4` (`:86-87`): `(12, 8, 12, 8)`.
pub fn panel_inner(panel: &[f32; 4]) -> [f32; 4] {
    let p = inner_panel(panel);
    uistate::inner(p, [12.0, 8.0, 12.0, 8.0])
}

/// The top strip — `uIElement2` (`:88-96`): `Width (0, 1)`, `Height 24`,
/// `VAlign 0`, padding 0, inside the panel's inner rect.
pub fn top_strip(panel: &[f32; 4]) -> [f32; 4] {
    let inner = panel_inner(panel);
    [inner[0], inner[1], inner[2], 24.0]
}

/// The middle container — `uIElement3` (`:113-122`):
/// `Height (-24 - 6 - 20, 1)`, `VAlign 1`, `Top -20`, padding 0.
pub fn middle_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = panel_inner(panel);
    let h = inner[3] - 24.0 - 6.0 - 20.0;
    [inner[0], inner[1] + inner[3] - 20.0 - h, inner[2], h]
}

/// The bestiary space — `uIElement5` (`:132-140`):
/// `Width (-12 - 230, 1)` (the info page's 230 px,
/// `UIBestiaryEntryInfoPage.cs:30`), `Height (-4, 1)`, `VAlign 1`,
/// padding 0, inside the middle container. The grid fills it
/// (`UIBestiaryEntryGrid` ctor `:23-24`).
pub fn bestiary_space(panel: &[f32; 4]) -> [f32; 4] {
    let mid = middle_rect(panel);
    let h = mid[3] - 4.0;
    [mid[0], mid[1] + mid[3] - h, mid[2] - 12.0 - 230.0, h]
}

/// The info page's rect — `UIBestiaryEntryInfoPage` (`:97-101` +
/// `UIBestiaryEntryInfoPage.cs:30-31`): `Width 230`, `Height (12, 1)`,
/// `HAlign 1` inside the middle container (`:145`). The page is ALWAYS
/// present beside the grid; an empty selection leaves its list empty
/// (`FillInfoForEntry(null)`, `:492-495`).
pub fn info_page_rect(panel: &[f32; 4]) -> [f32; 4] {
    let mid = middle_rect(panel);
    [mid[0] + mid[2] - 230.0, mid[1], 230.0, mid[3] + 12.0]
}

/// `GetEntriesToShow` (`UIBestiaryEntryGrid.cs:79-85`): the space divided
/// by 72 — no row growth, no minimum; the roster pages instead
/// (`OffsetLibraryByPages`). Zero-sized axes stay zero (the C# has no
/// `.max(1)` — an empty space shows no entries).
pub fn grid_metrics(rect: &[f32; 4]) -> (usize, usize) {
    ((rect[2] / CELL) as usize, (rect[3] / CELL) as usize)
}

/// The working-set slot under the cursor — the raw CELL lattice over the
/// bestiary space (the draw's placement; `grid_cell_rect` pins the C#'s
/// fractional form, exact when the space divides by 72). Returns the slot
/// index `0..cols*rows`, NOT the entry index — the caller maps it through
/// the page start (`_atEntryIndex`) and the working set.
pub fn grid_slot_at(screen: Vec2, mouse: Vec2) -> Option<usize> {
    let panel = panel_rect(screen);
    let rect = bestiary_space(&panel);
    let (cols, rows) = grid_metrics(&rect);
    if cols == 0 || rows == 0 {
        return None;
    }
    if mouse.x < rect[0] || mouse.y < rect[1] {
        return None;
    }
    let cx = ((mouse.x - rect[0]) / CELL) as usize;
    let cy = ((mouse.y - rect[1]) / CELL) as usize;
    if cx >= cols || cy >= rows {
        return None;
    }
    Some(cy * cols + cx)
}

/// `UIBestiaryEntryGrid.GetRangeText` (`UIBestiaryEntryGrid.cs:87-93`):
/// `"{min(at + 1, last)}-{last} ({total})"` where
/// `last = min(total, at + capacity)`. The empty roster prints `"0-0 (0)"`
/// (`Math.Min(1, 0)` is 0).
pub fn grid_range_text(at: usize, capacity: usize, total: usize) -> String {
    let last = total.min(at + capacity);
    format!("{}-{} ({})", (at + 1).min(last), last, total)
}

/// `UIBestiaryEntryGrid.FixBestiaryRange` (`UIBestiaryEntryGrid.cs:116-123`):
/// `Utils.Clamp(at + offset, 0, max(0, total - capacity))` — the page index
/// never runs past the last full page, and a roster smaller than one page
/// pins to 0. `OffsetLibraryByPages` (`:103-107`) passes
/// `pages * capacity` as the offset.
pub fn fix_grid_range(at: usize, offset: i64, capacity: usize, total: usize) -> usize {
    let max = total.saturating_sub(capacity);
    (at as i64 + offset).clamp(0, max as i64) as usize
}

// --- The element classes' missing pure halves (W4-d81 census rows 7-9) ---------
//
// The grid draw/hit/range/clamp are pre-landed above; the card draw
// (`draw_card`) covers the ctor's layer stack. These port the remaining
// stateless pieces the screens adopt later (the 18b convention).

/// `UIBestiaryEntryGrid.OffsetLibraryByPages` (`UIBestiaryEntryGrid.cs:103-107`):
/// the page buttons pass `howManyPages * maxEntriesToHave` to `OffsetLibrary`
/// (`:109-114`), which clamps through `FixBestiaryRange` and refills.
pub fn offset_library_by_pages(at: usize, pages: i64, capacity: usize, total: usize) -> usize {
    fix_grid_range(at, pages * capacity as i64, capacity, total)
}

/// `FillBestiarySpaceWithEntries`' cell placement
/// (`UIBestiaryEntryGrid.cs:50-66`): the button is 72x72, `HAlign/VAlign 0.5`,
/// `Left (0, k/cols - 0.5 + 0.5/cols)`, `Top (0, j/rows - 0.5 + 0.5/rows)` —
/// the fractional offsets centre the lattice inside the grid rect even when
/// the space is not a whole multiple of 72. The port's draw/hit use the raw
/// CELL lattice (the documented simplification, exact when the space divides).
pub fn grid_cell_rect(
    rect: &[f32; 4],
    cols: usize,
    rows: usize,
    col: usize,
    row: usize,
) -> [f32; 4] {
    let num3 = 0.5 / cols as f32; // `:51`
    let num4 = 0.5 / rows as f32; // `:52`
    let left_pct = col as f32 / cols as f32 - 0.5 + num3; // `:65`
    let top_pct = row as f32 / rows as f32 - 0.5 + num4; // `:66`
    [
        rect[0] + (rect[2] - CELL) * 0.5 + left_pct * rect[2],
        rect[1] + (rect[3] - CELL) * 0.5 + top_pct * rect[3],
        CELL,
        CELL,
    ]
}

/// `UIBestiaryEntryButton`'s border layer order (`MouseOver`/`MouseOut`,
/// `UIBestiaryEntryButton.cs:150-169`): hovered draws `Slot_Front` then
/// `Slot_Selection` (the overlay is removed, `:153-157`); idle draws
/// `Slot_Overlay` then `Slot_Front` (no glow, `:163-167`). Returns
/// `(draw_overlay, draw_glow)`. The pretty-portrait ctor removes the overlay
/// outright (`:93-96`). The current `draw_card` keeps the overlay under the
/// hover stack — a documented deviation until the adoption.
pub fn entry_button_border_layers(hovered: bool, pretty_portrait: bool) -> (bool, bool) {
    if pretty_portrait {
        return (false, false); // `:93-96` — the overlay never draws
    }
    if hovered {
        (false, true) // `:153-157`
    } else {
        (true, false) // `:163-167`
    }
}

/// `TryGettingBackgroundImageProvider` (`UIBestiaryEntryButton.cs:104-129`):
/// the first preference-matched provider with a non-null background image
/// wins (`:112-119`); failing that, the first provider with one in entry
/// order (`:120-127`). The port models each provider as
/// `(matches_a_preference, has_image)`; the bestiary entries carry no
/// background providers yet (the pretty-portrait backgrounds defer).
#[allow(dead_code)] // no background providers on the port's entries yet
pub fn pick_background_provider(providers: &[(bool, bool)]) -> Option<usize> {
    for (i, &(matched, has_image)) in providers.iter().enumerate() {
        if matched && has_image {
            return Some(i); // `:112-119`
        }
    }
    for (i, &(_, has_image)) in providers.iter().enumerate() {
        if has_image {
            return Some(i); // `:120-127`
        }
    }
    None // `:128`
}

/// `TryGettingDisplayIndex` (`UIBestiaryEntryButton.cs:131-140`): the first
/// `IBestiaryEntryDisplayIndex` in `entry.Info` supplies the corner number
/// (`None` = no number drawn, `:61-70`). The port's entries print their
/// roster position instead (`draw_card`); this is the source's lookup shape.
pub fn first_display_index(info_display_indices: &[Option<i32>]) -> Option<i32> {
    info_display_indices.iter().find_map(|x| *x) // `FirstOrDefault` (`:134`)
}

// --- the biome portrait backdrops (W4-e23 remainder) ----------------------------

/// The world-evil preference rows (`BestiaryDatabaseNPCsPopulator.cs:634`/
/// `:636`): the Desert Djinn 533 and the Tomb Crawler 529 wrap a
/// `BestiaryPortraitBackgroundBasedOnWorldEvilProviderPreferenceInfoElement`
/// (CorruptUndergroundDesert, CrimsonUndergroundDesert) instead of the plain
/// preference wrap. `Matches` picks the corrupt provider when
/// `!WorldGen.crimson`, the crimson one otherwise
/// (`BestiaryPortraitBackgroundBasedOnWorldEvilProviderPreferenceInfoElement.cs:22-29`).
/// The port persists no world-evil type (the `shop.rs`/`loot.rs` gap), so
/// the pick reads the corrupt arm — `WorldGen.crimson == false`, the same
/// default the fresh `Main` the C# samples carries.
const WORLD_EVIL_PREFERENCE: &[(i32, u32, u32)] = &[
    (533, 30, 35), // CorruptUndergroundDesert / CrimsonUndergroundDesert
    (529, 30, 35),
];

/// The Moon Lord's own background provider
/// (`MoonLordPortraitBackgroundProviderBestiaryInfoElement.cs:10-18`, added
/// at `BestiaryDatabaseNPCsPopulator.cs:639`): `MapBG1` at `Color.Black`.
/// It is a plain (non-preferred) provider appended last, so it supplies the
/// portrait COLOR even when a preferred tag won the image
/// (`NPCPortraitInfoElement.cs:71-75`).
const MOON_LORD_NET_ID: i32 = 398;

/// The portrait backdrop pick (`NPCPortraitInfoElement.ProvideUIElement`,
/// `NPCPortraitInfoElement.cs:36-84`): the first PREFERENCE-matched provider
/// with a background image wins the image (`:43-57`); failing that the first
/// provider with one in entry order (`:58-70`). The portrait COLOR is the
/// LAST provider (in entry order) that carries a `GetBackgroundColor`,
/// regardless of which image won (`:71-75`) — White when none does.
/// Returns `(sheet_name, tint)`; `None` when the entry draws no backdrop
/// (the `Slot_Back` alone, the pre-remainder state).
///
/// The port's providers are the entry's tag rows (the generated
/// `bestiary_tags` bg/bg_color data) plus the Moon Lord's own provider; the
/// preference wraps are the `preferred` flags (and the two world-evil rows).
/// The decorative overlays (`Visuals.*`) and the `SpawnConditionBestiaryOverlayInfoElement`
/// rows carry no background image and are skipped here (the overlay draw is
/// a separate, still-deferred layer — `UIBestiaryNPCEntryPortrait.cs:42-55`).
pub fn portrait_backdrop(net_id: i32) -> Option<(&'static str, [u8; 4])> {
    let tags = crate::bestiary_tags::entry_tags(net_id);
    // The preference-matched tag ids, in entry order. A plain `preferred`
    // row matches its own tag; a world-evil row matches the corrupt arm
    // (the port's `WorldGen.crimson == false` default).
    let evil = WORLD_EVIL_PREFERENCE.iter().find(|&&(id, _, _)| id == net_id);
    let matched = |t: &crate::bestiary_tags::EntryTag| -> bool {
        if let Some(&(_, corrupt, _crimson)) = evil {
            // The world-evil wrap is a `preferred` row too; it matches only
            // its corrupt provider tag.
            t.preferred && t.tag == corrupt
        } else {
            t.preferred
        }
    };
    // Pass 1: the first matched provider with an image (`:43-57`).
    let mut image: Option<&'static str> = None;
    for t in tags {
        if matched(t) {
            if let Some(bg) = crate::bestiary_tags::TAGS[t.tag as usize].bg {
                image = Some(bg);
                break;
            }
        }
    }
    let matched_won = image.is_some();
    // Pass 2: every provider in entry order — the first image when no
    // matched one won (`:67-70`), and the LAST background color (`:71-75`).
    let mut color: Option<[u8; 4]> = None;
    for t in tags {
        let def = &crate::bestiary_tags::TAGS[t.tag as usize];
        if let Some(bg) = def.bg {
            if !matched_won && image.is_none() {
                image = Some(bg);
            }
            if let Some(c) = def.bg_color {
                color = Some(c);
            }
        }
    }
    // The Moon Lord's own provider (appended last, `:639`): MapBG1 + Black.
    if net_id == MOON_LORD_NET_ID {
        if !matched_won && image.is_none() {
            image = Some("MapBG1");
        }
        color = Some([0, 0, 0, 255]); // `GetBackgroundColor` = Color.Black
    }
    image.map(|sheet| (sheet, color.unwrap_or([255, 255, 255, 255])))
}

/// `UIBestiaryEntryIcon`'s hover gate (`UIBestiaryEntryIcon.cs:38`, `:52`):
/// `IsMouseHovering || ForceHover` — the button's `MouseOver` sets
/// `ForceHover` on its icon (`UIBestiaryEntryButton.cs:158`/`:168`) because
/// the icon itself ignores mouse interaction (`:24`).
pub fn icon_is_hovered(is_mouse_hovering: bool, force_hover: bool) -> bool {
    is_mouse_hovering || force_hover
}

/// The icon's walk gate (`UnlockableNPCEntryIcon.cs:100-109`): without a
/// `Frame` override, a portrait OR hovered icon runs
/// `velocity.X = direction * Velocity` + `FindFrame` (the walk cycle); with
/// one, the frame pins to `frame.Height * num` (`:105-109`). The port's
/// immediate draw rests on frame 0 (no NPC cache to simulate — the documented
/// deviation); this pins the gate for the animation's landing.
pub fn icon_walks(frame_override: Option<i32>, is_portrait: bool, is_hovered: bool) -> bool {
    frame_override.is_none() && (is_portrait || is_hovered) // `:100`
}

// --- UIBestiaryEntryInfoPage (W4-d81 census row 10) ------------------------------
//
// The port's info page draws its rows directly (`draw_info_page`); these port
// the C# page's list-building state machine for the adoption.

/// The ctor's panel colours (`UIBestiaryEntryInfoPage.cs:33-34`): border
/// `(89, 116, 213)`, background `(73, 94, 171)` — no opacity multiply (the
/// power strip's 0.9 pair is a different panel).
pub fn info_page_colors() -> ([u8; 4], [u8; 4]) {
    ([89, 116, 213, 255], [73, 94, 171, 255])
}

/// The ctor's list insets (`UIBestiaryEntryInfoPage.cs:40-45`):
/// `SetPadding(2)` with `PaddingTop/Bottom 4`, `ListPadding 4`. Returns
/// `(left, top, right, bottom, list_padding)`.
pub const INFO_PAGE_LIST_PADDING: [f32; 5] = [2.0, 4.0, 2.0, 4.0, 4.0];

/// The scrollbar's box (`UIBestiaryEntryInfoPage.cs:47-52`): `Height (-20, 1)`,
/// `HAlign 1`, `VAlign 0.5`, `Left (-6, 0)` inside the 230 px page — 20 px
/// shorter than the page, centred vertically, 6 px in from the right edge.
pub fn info_page_scrollbar_rect(page: [f32; 4], scrollbar_w: f32) -> [f32; 4] {
    let h = page[3] - 20.0;
    [
        page[0] + page[2] - scrollbar_w - 6.0,
        page[1] + (page[3] - h) * 0.5,
        scrollbar_w,
        h,
    ]
}

/// `UpdateScrollbar` (`UIBestiaryEntryInfoPage.cs:59-65`):
/// `ViewPosition -= scrollWheelValue`.
pub fn info_page_scroll(view_position: f32, wheel: f32) -> f32 {
    view_position - wheel
}

/// `UIList.RecalculateChildren`' row stack (`UIList.cs:116-128`): each row
/// sits at the running height, `ListPadding` between rows (a single row
/// takes no padding, `:122`). Returns `(row_tops, total_height)` — the
/// total is `UIList.GetTotalHeight` (`:63-66`), the scrollbar's
/// `maxViewSize` (`UIList.UpdateScrollbar:130-137`).
pub fn info_page_row_layout(heights: &[f32], list_padding: f32) -> (Vec<f32>, f32) {
    let mut tops = Vec::with_capacity(heights.len());
    let mut y = 0.0;
    for (i, &h) in heights.iter().enumerate() {
        tops.push(y);
        let pad = if heights.len() == 1 { 0.0 } else { list_padding }; // `:122`
        y += h + pad; // `:125`
    }
    (tops, y)
}

/// The info page's list view height — the page's inner height after the
/// `(2, 4, 2, 4)` list padding (`UIBestiaryEntryInfoPage.cs:40-42`): the
/// scrollbar's `viewSize` (`UIList.UpdateScrollbar` reads
/// `GetInnerDimensions().Height`, `UIList.cs:134`).
pub fn info_page_view_height(page: [f32; 4]) -> f32 {
    page[3] - INFO_PAGE_LIST_PADDING[1] - INFO_PAGE_LIST_PADDING[3]
}

/// `UIScrollbar.SetView` (`UIScrollbar.cs:72-78`): the view clamps into
/// `[0, maxViewSize]`, the position into `[0, maxViewSize - viewSize]`.
/// Returns `(view, position)`.
pub fn info_page_set_view(view: f32, max_view: f32, position: f32) -> (f32, f32) {
    let view = view.clamp(0.0, max_view); // `:74`
    let position = position.clamp(0.0, max_view - view); // `:75`
    (view, position)
}

/// The info-page scrollbar's press (`UIScrollbar.LeftMouseDown`,
/// `UIScrollbar.cs:126-143`) — the shared [`super::panel`] helpers over
/// the page's bar rect. Returns `(new position, drag latch)`; `None` when
/// the press is off the bar. The bar is always attached (`CheckScrollBar`
/// forces `canScroll`, `:95`), so there is no detach gate here.
pub fn info_page_scrollbar_press(
    bar: [f32; 4],
    view: f32,
    max_view: f32,
    position: f32,
    mouse: Vec2,
) -> Option<(f32, Option<f32>)> {
    if !super::in_rect(mouse, bar) {
        return None;
    }
    let handle = super::panel::scrollbar_handle_at(bar, view, max_view, position);
    if super::in_rect(mouse, handle) {
        Some((position, Some(mouse.y - handle[1]))) // `:134-135`
    } else {
        Some((
            super::panel::scrollbar_jump_value(bar, view, max_view, handle[3], mouse.y),
            None,
        )) // `:137-141`
    }
}

/// `CheckScrollBar` (`UIBestiaryEntryInfoPage.cs:90-109`): the attach/detach
/// state machine — attached + !canScroll detaches (list width back to
/// `(0, 1)`), detached + canScroll attaches (list width `(-20, 1)`). Note the
/// C# forces `canScroll = true` (`:95`), so the detach arm is dead in the
/// source; the port keeps the machine faithful with the real flag. Returns
/// `(attached, list_width_px_offset)`.
pub fn info_page_check_scrollbar(attached: bool, can_scroll: bool) -> (bool, f32) {
    if attached && !can_scroll {
        (false, 0.0) // `:96-101`
    } else if !attached && can_scroll {
        (true, -20.0) // `:102-107`
    } else {
        (attached, if attached { -20.0 } else { 0.0 })
    }
}

/// `BestiaryInfoCategory` (`UIBestiaryEntryInfoPage.cs:11-20`) — the group
/// order `AddInfoToList` sorts by (`orderby x.Key`, `:131-133`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)] // the immediate draw pins its own row order
pub enum BestiaryInfoCategory {
    Nameplate,
    Portrait,
    FlavorText,
    Stats,
    ItemsFromCatchingNpc,
    ItemsFromDrops,
    Misc,
}

/// `GetBestiaryInfoCategory` (`UIBestiaryEntryInfoPage.cs:174-201`): the
/// element-type dispatch, in C# if-order. The port models the element kinds
/// as this enum's input tags (the info-element trait system is not ported).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)] // the immediate draw pins its own row order
pub enum BestiaryInfoElementKind {
    NpcPortrait,
    FlavorText,
    NamePlate,
    ItemFromCatchingNpc,
    ItemDrop,
    NpcStatsReport,
    NpcKillCounter,
    Other,
}

/// The category dispatch (`:174-201`).
#[allow(dead_code)] // the immediate draw pins its own row order
pub fn bestiary_info_category(kind: BestiaryInfoElementKind) -> BestiaryInfoCategory {
    use BestiaryInfoCategory as C;
    use BestiaryInfoElementKind as K;
    match kind {
        K::NpcPortrait => C::Portrait,                     // `:176-179`
        K::FlavorText => C::FlavorText,                    // `:180-183`
        K::NamePlate => C::Nameplate,                      // `:184-187`
        K::ItemFromCatchingNpc => C::ItemsFromCatchingNpc, // `:188-191`
        K::ItemDrop => C::ItemsFromDrops,                  // `:192-195`
        K::NpcStatsReport | K::NpcKillCounter => C::Stats, // `:196-199`
        K::Other => C::Misc,                               // `:200`
    }
}

/// `AddInfoToList`'s ordering (`UIBestiaryEntryInfoPage.cs:128-163`): group by
/// category (ascending), within a group order by `OrderPriority` DESCENDING
/// (`GetIndividualElementPriority`, `:165-172` — 0 for non-prioritized
/// elements; `OrderByDescending` is stable, so ties keep entry order). A
/// separator follows every non-empty group (`:151-160`) except the last
/// (`_list.Remove(item)`, `:162`). Input: `(kind, priority, produced_a_row)`
/// per info element in entry order; output: the row indices in draw order
/// plus the separator positions (as insert-after indices into that order).
#[allow(dead_code)] // the immediate draw pins its own row order
pub fn info_page_row_order(
    elements: &[(BestiaryInfoElementKind, f32, bool)],
) -> (Vec<usize>, Vec<usize>) {
    let mut groups: Vec<(BestiaryInfoCategory, Vec<usize>)> = Vec::new();
    for (i, &(kind, _, produced)) in elements.iter().enumerate() {
        if !produced {
            continue; // `ProvideUIElement` returned null (`:144-149`)
        }
        let cat = bestiary_info_category(kind);
        match groups.iter_mut().find(|(c, _)| *c == cat) {
            Some((_, rows)) => rows.push(i),
            None => groups.push((cat, vec![i])),
        }
    }
    groups.sort_by_key(|(c, _)| *c); // `orderby x.Key` (`:132`)
    let mut order = Vec::new();
    let mut separators = Vec::new();
    let n_groups = groups.len();
    for (g, (_, rows)) in groups.into_iter().enumerate() {
        // Stable descending priority (`:142`).
        let mut rows = rows;
        rows.sort_by(|&a, &b| elements[b].1.partial_cmp(&elements[a].1).unwrap());
        order.extend(rows);
        if g + 1 < n_groups {
            separators.push(order.len()); // the separator after this group
        }
    }
    (order, separators)
}

// --- UIBestiaryFilteringOptionsGrid (W4-d81 census row 11) -----------------------

/// `GetDisplaySettings` (`UIBestiaryFilteringOptionsGrid.cs:159-173`): the
/// 32x32 buttons on a 34 px pitch (`num = 2`), `perRow` hard-overwritten to
/// 12 (`:167` — the sqrt walk is dead), `howManyRows = ceil(count / 12)`,
/// the offsets hard-overwritten to 6 (`:171-172` — the centring walk dead).
/// Returns `(per_row, rows, offset_left, offset_top)`.
pub fn filter_grid_display_settings(count: usize) -> (usize, usize, f32, f32) {
    let per_row = 12usize; // `:167`
    let rows = count.div_ceil(per_row); // `:168`
    (per_row, rows, 6.0, 6.0) // `:171-172`
}

/// `BuildContainer`'s panel size (`UIBestiaryFilteringOptionsGrid.cs:46-56`):
/// `Width = perRow * 34 + 10`, `Height = rows * 34 + 10`, `HAlign 1`,
/// border/background the power-strip pair `(89,116,213)*0.9` /
/// `(73,94,171)*0.9`. Returns `(width, height)`.
pub fn filter_grid_container_size(count: usize) -> [f32; 2] {
    let (per_row, rows, _, _) = filter_grid_display_settings(count);
    [per_row as f32 * 34.0 + 10.0, rows as f32 * 34.0 + 10.0]
}

/// Filter button `i`'s top-left inside the container
/// (`UIBestiaryFilteringOptionsGrid.cs:90-102`):
/// `Top = 6 + (i / 12) * 34`, `Left = 6 + (i % 12) * 34`.
pub fn filter_button_top_left(i: usize) -> [f32; 2] {
    let (_, _, left, top) = filter_grid_display_settings(0);
    [left + (i % 12) as f32 * 34.0, top + (i / 12) as f32 * 34.0]
}

/// `UpdateButtonSelections` (`UIBestiaryFilteringOptionsGrid.cs:175-190`):
/// active filters take `(152, 175, 235)` at opacity 1, inactive the
/// `InventoryDefaultColor` at 0.7; `SetCurrentOption(active ? i : -1)` drives
/// `IsSelected`. Returns `(color, opacity, current_option)`.
pub fn filter_button_selection(active: bool, option: i32) -> ([u8; 4], f32, i32) {
    if active {
        ([152, 175, 235, 255], 1.0, option) // `:181-184`
    } else {
        (uistate::INVENTORY_DEFAULT_COLOR, 0.7, -1) // `:185-188`
    }
}

/// `GetIsFilterAvailableForEntries` (`UIBestiaryFilteringOptionsGrid.cs:192-207`):
/// `ForcedDisplay` wins when set (`:194-198`); else any entry that fits the
/// filter AND is known past tier 0 (`:199-205`). The port models the test
/// list as `(fits, unlock_state_above_zero)` pairs.
pub fn filter_is_available(forced: Option<bool>, tests: &[(bool, bool)]) -> bool {
    if let Some(f) = forced {
        return f; // `:194-198`
    }
    tests.iter().any(|&(fits, known)| fits && known) // `:199-206`
}

/// The unavailable button's state (`UIBestiaryFilteringOptionsGrid.cs:118-146`):
/// option `-2`, `FadeFromBlack = 0.5`, the `Icon_Tags_Shadow` frame
/// `(16, 5, col 0, row 4)` at `White * 0.2`; the filter is removed from
/// `ActiveFilters` (`:120`). Returns `(option, fade_from_black, icon_tint)`.
pub fn unavailable_filter_button() -> (i32, f32, [u8; 4]) {
    (-2, 0.5, super::color::mul([255, 255, 255, 255], 0.2))
}

// --- UIBestiaryInfoItemLine (W4-d81 census row 12) -------------------------------
//
// `drop_rate_text` above ports `GetDropInfo`'s rate column; these are the
// line's remaining halves.

/// The ctor's geometry (`UIBestiaryInfoItemLine.cs:26-31`): `Width (-14, 1)`,
/// `Height 32`, `Left 5`, `PaddingLeft/Right 10`, `SetPadding(0)` first.
/// Returns the line rect inside the list's content box.
pub fn info_item_line_rect(content: [f32; 4]) -> [f32; 4] {
    [content[0] + 5.0, content[1], content[2] - 14.0, 32.0]
}

/// The locked line's icon (`UIBestiaryInfoItemLine.cs:36-55`): below tier 3
/// the whole line is the `Icon_Locked` texture at `ImageScale 0.55`, centred
/// (`HAlign/VAlign 0.5` in a full-size container). Returns the drawn rect for
/// a texture of `tex_size` centred in `line`.
pub fn info_item_locked_icon_rect(line: [f32; 4], tex_size: [f32; 2]) -> [f32; 4] {
    let (w, h) = (tex_size[0] * 0.55, tex_size[1] * 0.55);
    [
        line[0] + (line[2] - w) * 0.5,
        line[1] + (line[3] - h) * 0.5,
        w,
        h,
    ]
}

/// The border colour pair (`UIBestiaryInfoItemLine.cs:34`, `MouseOver`/
/// `MouseOut` `:158-167`): idle `(89, 116, 213)`, hovered
/// `Colors.FancyUIFatButtonMouseOver`. The hover also fires sound 12
/// (`:160` — the shared `elements.rs::hover_sound_tick` latch).
pub fn info_item_line_border(hovered: bool) -> [u8; 4] {
    if hovered {
        uistate::FANCY_UI_FAT_BUTTON_MOUSE_OVER // `:161`
    } else {
        [89, 116, 213, 255] // `:34`/`:166`
    }
}

/// `SetBestiaryNotesOnItemCache` (`UIBestiaryInfoItemLine.cs:137-156`): the
/// non-null, non-whitespace condition descriptions joined with `"\n"`; a
/// null condition list leaves the notes untouched (the port returns the
/// empty string — the item cache has no notes field yet).
pub fn bestiary_notes(conditions: &[Option<&str>]) -> String {
    let mut list = Vec::new();
    for c in conditions {
        if let Some(desc) = c {
            if !desc.trim().is_empty() {
                list.push(*desc); // `:148-152`
            }
        }
    }
    list.join("\n") // `:155`
}

/// `DrawSelf`'s hover gate (`UIBestiaryInfoItemLine.cs:112-119`): the item
/// tooltip (`Main.HoverItem`, `:121-126`) shows only while hovered AND the
/// line is not the locked variant (`_hideMouseOver`, `:38`).
#[allow(dead_code)] // the drop-row hover tooltip lands with the tooltip work
pub fn info_item_shows_tooltip(hovered: bool, hide_mouse_over: bool) -> bool {
    hovered && !hide_mouse_over
}

// --- UIBestiaryInfoLine (W4-d81 census row 13) -----------------------------------
//
// `compare_manual_order` (elements.rs, batch 1) ports `CompareTo` (`:92-99`).

/// `SetText` (`UIBestiaryInfoLine.cs:73-81`): the measured size is
/// `(MouseText.MeasureString(text).X, 16) * textScale`; the element's
/// `MinWidth/MinHeight` are the size plus the padding pair. Returns
/// `(text_size, min_width, min_height)` for a measured text width and the
/// four paddings.
#[allow(dead_code)] // the immediate stat rows keep their own pitch
pub fn info_line_text_size(
    measured_w: f32,
    text_scale: f32,
    padding: [f32; 4],
) -> ([f32; 2], f32, f32) {
    let size = [measured_w * text_scale, 16.0 * text_scale]; // `:75`
    (
        size,
        size[0] + padding[0] + padding[2], // `MinWidth` (`:79`)
        size[1] + padding[1] + padding[3], // `MinHeight` (`:80`)
    )
}

/// `DrawSelf`'s text position (`UIBestiaryInfoLine.cs:83-90`):
/// `pos = inner.Position()`, `Y -= 2 * textScale`, `X += (inner.Width -
/// textSize.X) * 0.5` — the line centres its text horizontally and rides
/// 2 px up per scale unit.
#[allow(dead_code)] // the immediate stat rows keep their own placement
pub fn info_line_text_pos(inner: [f32; 4], text_size: [f32; 2], text_scale: f32) -> [f32; 2] {
    [
        inner[0] + (inner[2] - text_size[0]) * 0.5, // `:88`
        inner[1] - 2.0 * text_scale,                // `:87`
    ]
}

// --- UIBestiaryNPCEntryPortrait (W4-d81 census row 14) ---------------------------
//
// The info-page portrait card (`draw_info_page`) draws the C# element's
// 193x112 geometry (W4-e23 adopted it).

/// The ctor's size (`UIBestiaryNPCEntryPortrait.cs:17-18`): 193x112,
/// `SetPadding(0)`.
pub const NPC_PORTRAIT_SIZE: [f32; 2] = [193.0, 112.0];

/// The inner container (`UIBestiaryNPCEntryPortrait.cs:20-28`):
/// `Width/Height (-4, 1)`, centred, `OverflowHidden` — the same 2 px inset
/// box the entry button uses, over the 193x112 card.
pub fn npc_portrait_inner(card: [f32; 4]) -> [f32; 4] {
    [card[0] + 2.0, card[1] + 2.0, card[2] - 4.0, card[3] - 4.0]
}

/// The overlay tint fallback (`UIBestiaryNPCEntryPortrait.cs:53`): an
/// overlay without its own colour takes `Color.Lerp(Color.White,
/// portraitColor, 0.5)`.
#[allow(dead_code)] // the port's entries carry no background overlays yet
pub fn npc_portrait_overlay_color(
    overlay_color: Option<[u8; 4]>,
    portrait_color: [u8; 4],
) -> [u8; 4] {
    overlay_color.unwrap_or_else(|| {
        super::char_creation::lerp_rgb([255, 255, 255, 255], portrait_color, 0.5)
    })
}

// --- UIBestiarySortingOptionsGrid (W4-d81 census row 15) -------------------------

/// `BuildGrid`'s container (`UIBestiarySortingOptionsGrid.cs:37-61`):
/// `num = 2`, `num2 = 28` pitch; the panel is `Width 126`,
/// `Height = visible_count * 28 + 5 + 3`, `HAlign 1`, `Left -118`. Only
/// steps with `!HiddenFromSortOptions` count (`:42-48`). Returns
/// `(width, height)`.
pub fn sort_grid_container_size(visible_count: usize) -> [f32; 2] {
    [126.0, visible_count as f32 * 28.0 + 5.0 + 3.0] // `:51-52`
}

/// Sort button `n`'s rect inside the container
/// (`UIBestiarySortingOptionsGrid.cs:68-74`): `Width 114`, `Height 26`
/// (`num2 - num`), `HAlign 0.5`, `Top = 5 + 28 * n`.
pub fn sort_button_rect(container: [f32; 4], n: usize) -> [f32; 4] {
    [
        container[0] + (container[2] - 114.0) * 0.5,
        container[1] + 5.0 + 28.0 * n as f32,
        114.0,
        26.0,
    ]
}

/// `ClickOption` (`UIBestiarySortingOptionsGrid.cs:89-115`): clicking the
/// already-selected step falls back to `_defaultStepIndex` (`:92-95`);
/// the clicked step takes `(152, 175, 235)` at opacity 1, the rest the
/// `InventoryDefaultColor` at 0.7 (`:96-108` — the same pair the filtering
/// grid's `UpdateButtonSelections` uses). Returns the new selected index.
pub fn sort_click_option(clicked: i32, current_selected: i32, default_step: i32) -> i32 {
    if clicked == current_selected {
        default_step // `:92-95`
    } else {
        clicked
    }
}

/// The sort button's colour pair (`UIBestiarySortingOptionsGrid.cs:100-107`)
/// — the C# duplicates the filtering grid's `UpdateButtonSelections` walk
/// per class; the port shares the one helper (house style).
pub fn sort_button_colors(selected: bool) -> ([u8; 4], f32) {
    let (color, opacity, _) = filter_button_selection(selected, 0);
    (color, opacity)
}

/// `GetEntriesToShow` (`UIBestiarySortingOptionsGrid.cs:117-122`): one
/// column, one row per visible button.
pub fn sort_grid_entries_to_show(visible_count: usize) -> (usize, usize, usize) {
    (1, visible_count, visible_count)
}

/// The completion bar's rect — `FillProgressBottomBar`
/// (`UIBestiaryTest.cs:158-179`): the `UIColoredSliderSimple` fills the
/// bottom strip (`Width (0, 1)`, `Height 15`, `VAlign 1`, `:168-171`); the
/// strip itself is the panel's bottom 20 px row (`BuildPage`, `:123-131`),
/// `VAlign 1` inside the panel's inner rect (the 8 px bottom padding).
pub fn progress_bar_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = panel_inner(panel);
    [inner[0], inner[1] + inner[3] - 15.0, inner[2], 15.0]
}

/// `GetCompletionPercentText` (`UIBestiaryTest.cs:190-196`):
/// `Language.GetTextValueWith("BestiaryInfo.PercentCollected", { Percent =
/// Utils.PrettifyPercentDisplay(report.CompletionPercent, "P2") })` — the
/// "{Percent} Collected" row with the trimmed two-decimal share.
pub fn completion_percent_text(fraction: f32) -> String {
    crate::lang::text_args(
        "BestiaryInfo.PercentCollected",
        &[prettify_percent_display(fraction)],
    )
}

/// `NPCKillCounterInfoElement.ProvideUIElement`'s hover line
/// (`NPCKillCounterInfoElement.cs:90-96`): the kill count's tooltip is
/// `BestiaryInfo.Slain` ("Slain"), the count itself is the raw integer
/// (`:46`, `:68` — scale 0.85) beside the grave icon (item 321, `:75-77`).
/// `GetKillCount` (`:98-101`) returns null/0 before the first kill and the
/// element is skipped (`:26-30`).
pub fn kill_counter_text(kills: i32) -> Option<String> {
    if kills < 1 {
        return None;
    }
    Some(kills.to_string())
}

/// `NPCStatsReportInfoElement`'s knockback label
/// (`NPCStatsReportInfoElement.cs:123`): `> 0.8` High, `> 0.4` Med,
/// `<= 0` None, else Low (`BestiaryInfo.Knockback{High,Medium,None,Low}`).
/// Below tier 2 the whole stats block prints "???" (`:124-128`).
pub fn knockback_label(resist: f32, stats_known: bool) -> String {
    if !stats_known {
        return "???".to_string();
    }
    let key = if resist > 0.8 {
        "BestiaryInfo.KnockbackHigh"
    } else if resist > 0.4 {
        "BestiaryInfo.KnockbackMedium"
    } else if !(resist > 0.0) {
        "BestiaryInfo.KnockbackNone"
    } else {
        "BestiaryInfo.KnockbackLow"
    };
    crate::lang::text(key)
}

// --- NPCStatsReportInfoElement's 2x2 grid + coin row (W4-e23 remainder) ---------

/// The stat icons' natural size (`Stat_HP`/`Attack`/`Defense`/`Knockback`
/// are 96x31 sheets; `UIImage` keeps the texture size,
/// `UIImage.cs:52-56`).
pub const STAT_ICON_SIZE: [f32; 2] = [96.0, 31.0];

/// The coin icons' natural size (`Stat_Platinum/Gold/Silver/Copper`,
/// 47x31).
pub const COIN_ICON_SIZE: [f32; 2] = [47.0, 31.0];

/// `ProvideUIElement`'s 2x2 icon lattice (`NPCStatsReportInfoElement.cs:69-96`):
/// `num = 99` column pitch, `num2 = 35` row pitch, `num3 = 3` left inset,
/// `num4 = 0` top. Append order: HP (3, 0), Attack (3, 35),
/// Defense (102, 35), Knockback (102, 0). `origin` is the block's top-left.
pub fn stats_grid_rects(origin: [f32; 2]) -> [[f32; 4]; 4] {
    let (num, num2, num3, num4) = (99.0, 35.0, 3.0, 0.0); // `:69-72`
    [
        [origin[0] + num3, origin[1] + num4, STAT_ICON_SIZE[0], STAT_ICON_SIZE[1]], // HP `:73-77`
        [origin[0] + num3, origin[1] + num4 + num2, STAT_ICON_SIZE[0], STAT_ICON_SIZE[1]], // Attack `:78-82`
        [origin[0] + num3 + num, origin[1] + num4 + num2, STAT_ICON_SIZE[0], STAT_ICON_SIZE[1]], // Defense `:83-87`
        [origin[0] + num3 + num, origin[1] + num4, STAT_ICON_SIZE[0], STAT_ICON_SIZE[1]], // Knockback `:88-92`
    ]
}

/// The value `UIText`'s draw position inside a stat/coin icon
/// (`:129-160`, `:217-244`): `HAlign 1`, `VAlign 0.5`, `Left = left`
/// (-10 for the grid, -7 for the coins), scale `text_scale`. The element
/// is `text_w x 16*scale` (`UIText.InternalSetText:186` pins the
/// non-wrapped height at 16), centred vertically, then `DrawSelf` rides
/// `Y -= 2 * textScale` (`UIText.cs:138`).
pub fn stat_value_pos(icon: [f32; 4], text_w: f32, text_scale: f32, left: f32) -> [f32; 2] {
    [
        icon[0] + icon[2] - text_w + left,
        icon[1] + (icon[3] - 16.0 * text_scale) * 0.5 - 2.0 * text_scale,
    ]
}

/// The coin row's four denomination strings
/// (`NPCStatsReportInfoElement.cs:99-119`): `(int)MonetaryValue` split
/// into platinum/gold/silver/copper, each clamped (999/99/99/99), a
/// denomination whose division is `< 1` prints `"-"`. Below tier 2 (or
/// `HideStats`) all four become `"?"` (`:124-128`).
pub fn coin_strings(value: f32, stats_known: bool) -> [String; 4] {
    let num7 = value as i32; // `:99`
    let mut out = [
        (num7 / 1000000).clamp(0, 999).to_string(),
        (num7 % 1000000 / 10000).clamp(0, 99).to_string(),
        (num7 % 10000 / 100).clamp(0, 99).to_string(),
        (num7 % 100).clamp(0, 99).to_string(),
    ];
    if num7 / 1000000 < 1 {
        out[0] = "-".to_string(); // `:104-107`
    }
    if num7 / 10000 < 1 {
        out[1] = "-".to_string(); // `:108-111`
    }
    if num7 / 100 < 1 {
        out[2] = "-".to_string(); // `:112-115`
    }
    if num7 < 1 {
        out[3] = "-".to_string(); // `:116-119`
    }
    if !stats_known {
        out = ["?".to_string(), "?".to_string(), "?".to_string(), "?".to_string()]; // `:126`
    }
    out
}

/// The coin row's icon rects (`:176-198`): `num9 = num3 = 3` left,
/// `num10 = num8 + 8 = 78` top (the separator sits at 70, `num8` took
/// `+4` at `:175`), `num11 = 49` pitch — platinum 3, gold 52, silver
/// 102 (`+1`), copper 151 (`+1`). Only denominations whose string is not
/// `"-"` append their icon (`:199-214`); the caller gates the whole row
/// on `value > 0` (`:166`).
pub fn coin_row_rects(origin: [f32; 2]) -> [[f32; 4]; 4] {
    let (num9, num10, num11) = (3.0, 78.0, 49.0);
    [
        [origin[0] + num9, origin[1] + num10, COIN_ICON_SIZE[0], COIN_ICON_SIZE[1]],
        [origin[0] + num9 + num11, origin[1] + num10, COIN_ICON_SIZE[0], COIN_ICON_SIZE[1]],
        [origin[0] + num9 + num11 * 2.0 + 1.0, origin[1] + num10, COIN_ICON_SIZE[0], COIN_ICON_SIZE[1]],
        [origin[0] + num9 + num11 * 3.0 + 1.0, origin[1] + num10, COIN_ICON_SIZE[0], COIN_ICON_SIZE[1]],
    ]
}

/// The stats block's height (`:165`, `:249-251`): `num8 = 66`, `+4` for
/// the coin separator and `+34` for the coin row when `value > 0`, then
/// `Height.Pixels = num8 + 4` — 70 without coins, 108 with.
pub fn stats_block_height(has_coins: bool) -> f32 {
    let mut num8 = 66.0;
    if has_coins {
        num8 += 4.0; // `:175`
        num8 += 34.0; // `:249`
    }
    num8 + 4.0 // `:251`
}

/// `NPCKillCounterInfoElement.ProvideUIElement`'s panel
/// (`NPCKillCounterInfoElement.cs:43-65`): the row is 30 tall (`num3`),
/// the `Stat_Panel` panel `Width (-8 + num4, 1)` with `num4 = -3`
/// (`:51`), `Height 30`, `Left -8`, `HAlign 1`, bg `(43, 56, 101)`,
/// `PaddingRight 5` (`:65`). `content` is the list's content box,
/// `row_top` the row's y.
pub fn kill_counter_panel_rect(content: [f32; 4], row_top: f32) -> [f32; 4] {
    let w = content[2] - 8.0 + -3.0; // `:56`
    [content[0] + content[2] - 8.0 - w, row_top, w, 30.0] // `:61-62`
}


/// `UIBestiaryInfoItemLine.GetDropInfo`'s rate column
/// (`UIBestiaryInfoItemLine.cs:92-109`): tier 4 prints the stack range + the
/// prettified rate ("P4" under 0.1 %, "100%" at the full rate); tier 3
/// prints "???" and drops the stack range (`:105-109`). Below tier 3 the
/// whole line is the locked icon (`:36-55`), never a text row.
pub fn drop_rate_text(state: UnlockState, rate: f32, stack_min: i32, stack_max: i32) -> String {
    if state < UnlockState::CanShowDropsWithoutDropRates {
        return String::new();
    }
    let stack_range = if stack_min != stack_max {
        format!(" ({stack_min}-{stack_max})")
    } else if stack_min == 1 {
        String::new()
    } else {
        format!(" ({stack_min})")
    };
    let rate_text = if rate != 1.0 {
        prettify_percent_display(rate)
    } else {
        "100%".to_string()
    };
    if state != UnlockState::CanShowDropsWithDropRates {
        return "???".to_string();
    }
    // `:64-67`: the stack range prefixes the rate with one space.
    if stack_range.is_empty() {
        rate_text
    } else {
        format!("{stack_range} {rate_text}")
    }
}

/// `UIBestiaryEntryIcon.GetHoverText` → `UnlockableNPCEntryIcon.GetHoverText`
/// (`UIBestiaryEntryIcon.cs:69-72`, `UnlockableNPCEntryIcon.cs:209-221`):
/// `Lang.GetNPCNameValue(netID)` once the entry is known, "???" while it is
/// not — the card hover's `MouseText` (`UIBestiaryEntryButton.cs:142-148`).
pub fn entry_hover_text(entry: &crate::bestiary::BestiaryEntryDef, state: UnlockState) -> String {
    if !state.known() {
        return "???".to_string();
    }
    let localized = crate::lang::npc_name(entry.net_id);
    if localized.is_empty() {
        entry.name.to_string()
    } else {
        localized
    }
}

/// `BestiaryEntry.CommonEnemy`'s flavor-text key
/// (`BestiaryEntry.cs:46-50`): `"Bestiary_FlavorText.npc_" +
/// Lang.GetNPCName(netID).Key.Replace("NPCName.", "")`, added only when
/// `Language.Exists` — the port's credit id IS the `NPCName.<Const>` tail
/// (`ContentSamples.cs:862-873`). `FlavorTextBestiaryInfoElement` shows it
/// from tier 2 (`FlavorTextBestiaryInfoElement.cs:20-23`).
pub fn flavor_text(entry: &crate::bestiary::BestiaryEntryDef, state: UnlockState) -> Option<String> {
    if state < UnlockState::CanShowStats {
        return None;
    }
    let key = format!("Bestiary_FlavorText.npc_{}", entry.credit_id);
    if crate::lang::exists(&key) {
        Some(crate::lang::text(&key))
    } else {
        None
    }
}

/// `true` while the cursor is over the Back panel (`MakeExitButton`'s
/// `OnLeftMouseDown` -> `Click_GoBack`).
pub fn back_hit(screen: Vec2, mouse: Vec2) -> bool {
    let panel = panel_rect(screen);
    let r = back_rect(&panel);
    mouse.x >= r[0] && mouse.x <= r[0] + r[2] && mouse.y >= r[1] && mouse.y <= r[1] + r[3]
}

/// `true` while the cursor is over the inventory's bestiary button
/// (`Main.cs:42441`'s `rectangle.Contains(mouse)` — inclusive edges).
pub fn button_hit(mouse: Vec2) -> bool {
    mouse.x >= BUTTON_RECT[0]
        && mouse.x <= BUTTON_RECT[0] + BUTTON_RECT[2]
        && mouse.y >= BUTTON_RECT[1]
        && mouse.y <= BUTTON_RECT[1] + BUTTON_RECT[3]
}

/// `DrawBestiaryIcon` (`Main.cs:42423-42485`): the 2-frame `Images/UI/Bestiary`
/// sheet drawn frame-hover, inset 2 px per side, centred in the 30x30 rect.
pub fn draw_inventory_button(r: &mut SpriteRenderer, lookups: &Lookups, mouse: Vec2) {
    let Some(sheet) = lookups.ui_bestiary_button else {
        return;
    };
    // `value.Frame(2, 1, flag ? 1 : 0)`, `rectangle2.Width -= 2` (`:42464-42467`).
    draw_two_frame_button(r, lookups, &sheet, BUTTON_RECT, button_hit(mouse));
}

/// `DrawEmoteBubblesButton` (`Main.cs:42369-42408`): the same two-frame
/// button treatment at `EMOTE_BUTTON_RECT` — `TextureAssets.EmoteMenuButton`
/// (`AssetInitializer.cs:313`: `Images\UI\Emotes`), frame 1 while the
/// pointer is inside `rectangle` (`:42397`), inset 2 px per side
/// (`:42402-42406`). The draw call was missing entirely before §14: the
/// button had a hitbox and a click handler but no sprite.
pub fn draw_emote_button(r: &mut SpriteRenderer, lookups: &Lookups, mouse: Vec2) {
    let Some(sheet) = lookups.ui_emote_button else {
        return;
    };
    draw_two_frame_button(
        r,
        lookups,
        &sheet,
        EMOTE_BUTTON_RECT,
        emote_button_hit(mouse),
    );
}

/// The shared `value.Frame(2, 1, flag ? 1 : 0)` + the 2 px inset body of
/// `DrawBestiaryIcon` (`:42464-42467`) and `DrawEmoteBubblesButton`
/// (`:42397-42406`) — both sheets are horizontal 2-frame strips, so the
/// frame is a column slice, never a row one.
fn draw_two_frame_button(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    sheet: &tr_render::SheetRect,
    rect: [f32; 4],
    hover: bool,
) {
    let dst = two_frame_dst(sheet, rect);
    draw_sheet_frame(r, lookups, sheet, 2, 1, hover as usize, dst, [1.0; 4]);
}

/// Where a 2-frame button sheet lands inside its rect: the sheet's one
/// frame (`width / 2`, the full height) shrunk 2 px per side and centred
/// (`DrawBestiaryIcon`'s `rectangle2.Width -= 2; .Height -= 2`).
fn two_frame_dst(sheet: &tr_render::SheetRect, rect: [f32; 4]) -> [f32; 4] {
    let frame_w = (sheet.width / 2) as f32;
    let frame_h = sheet.height as f32;
    let cx = rect[0] + rect[2] * 0.5;
    let cy = rect[1] + rect[3] * 0.5;
    [
        cx - (frame_w - 2.0) * 0.5,
        cy - (frame_h - 2.0) * 0.5,
        frame_w - 2.0,
        frame_h - 2.0,
    ]
}

/// The whole screen: the dim backdrop, the container panel, the top strip
/// (page buttons, range panel, sort/filter toggles, search bar), the paged
/// entry grid beside the info page, the option grid overlay, the completion
/// bar and the Back panel. `ui` is the [`super::bestiary_screen::BestiaryUi`]
/// working-set state; `selected` indexes `ENTRIES` (the info page's entry).
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &WorldBestiary,
    ui: &super::bestiary_screen::BestiaryUi,
    selected: Option<usize>,
    mouse: Vec2,
) {
    // The state's background — the game keeps rendering behind
    // (`IngameFancyUI`), dimmed the way the port's other overlays are.
    r.push_ui_sprite([0.0, 0.0], [screen.x, screen.y], r.white_uv(), BACKDROP);
    draw_panel_chrome(r, lookups, screen);

    // The grid and the info page draw SIDE BY SIDE (`BuildPage:132-146`):
    // the space holds the grid, the 230 px page rides `HAlign 1` beside it
    // whether or not an entry is selected (`FillInfoForEntry(null)` empties
    // the list, `:492-495`).
    draw_grid(r, lookups, screen, tracker, ui, mouse);
    draw_info_page(r, lookups, screen, tracker, selected, mouse, ui.info_scroll);
    super::bestiary_screen_draw::draw_top_strip(r, lookups, screen, ui, mouse);
    // The option grid overlays the bestiary space (`:401-415`).
    super::bestiary_screen_draw::draw_option_grid(r, lookups, screen, tracker, ui, mouse);

    // The completion bar — `FillProgressBottomBar` (`UIBestiaryTest.cs:158-179`)
    // + `ShowStats_Completion`/`GetCompletionPercentText` (`:181-196`):
    // `GetUnlockProgress` (`:497-513`) counts every entry past
    // `NotKnownAtAll_0` over the ORIGINAL roster — NOT the working set and
    // NOT only the fully-unlocked ones; `CompletionPercent`
    // (`BestiaryUnlockProgressReport.cs:9-19`) is that share, 1 for an empty
    // roster. The bar: `UIColoredSliderSimple` 15 px tall, full width, bottom
    // of the 20 px bottom strip (`:167-175`), FilledColor (51, 137, 255),
    // EmptyColor (35, 43, 81); the percent label rides the hover
    // (`ShowStats_Completion` -> `MouseText`, `:181-188`).
    let panel = panel_rect(screen);
    let pct = crate::bestiary::completion_percent(tracker);
    let bar = progress_bar_rect(&panel);
    r.push_ui_sprite([bar[0], bar[1]], [bar[2], bar[3]], r.white_uv(), super::color::tint([35, 43, 81, 255]));
    r.push_ui_sprite(
        [bar[0], bar[1]],
        [bar[2] * pct.clamp(0.0, 1.0), bar[3]],
        r.white_uv(),
        super::color::tint([51, 137, 255, 255]),
    );
    if super::in_rect(mouse, bar) {
        // `Language.GetTextValueWith("BestiaryInfo.PercentCollected", {Percent})`
        // (`:192-195`) — "{Percent} Collected", the P2-prettified share.
        let label = completion_percent_text(pct);
        super::tooltip::draw_mouse_text(r, lookups, mouse, &label, screen, super::UiPulse::default());
    }

    // `MakeExitButton` (`:364-380`): the 0.7-large Back panel.
    let r0 = back_rect(&panel);
    let style = super::cards::fat_button_style();
    let back_label = crate::lang::text("UI.Back");
    let text_w = text::hud_text_width(r, lookups, &back_label, style.text_scale);
    super::uistate::draw_text_panel(
        r,
        lookups,
        r0,
        &style,
        &back_label,
        text_w,
        back_hit(screen, mouse),
    );
}

fn draw_panel_chrome(r: &mut SpriteRenderer, lookups: &Lookups, screen: Vec2) {
    // The panel body: `UIPanel`'s `BackgroundColor = (33, 43, 79) * 0.8`
    // (`UIBestiaryTest.cs:84-88`), `Height (-90, 1)` (`:83`) — drawn as a
    // flat quad (the port's panel style for the full-screen states).
    let p = inner_panel(&panel_rect(screen));
    r.push_ui_sprite(
        [p[0], p[1]],
        [p[2], p[3]],
        r.white_uv(),
        [
            33.0 / 255.0 * 0.8,
            43.0 / 255.0 * 0.8,
            79.0 / 255.0 * 0.8,
            0.8,
        ],
    );
    let _ = lookups;
}

fn draw_grid(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &WorldBestiary,
    ui: &super::bestiary_screen::BestiaryUi,
    mouse: Vec2,
) {
    let panel = panel_rect(screen);
    let rect = bestiary_space(&panel);
    let (cols, rows) = grid_metrics(&rect);
    if cols == 0 || rows == 0 {
        return;
    }
    let capacity = cols * rows;
    // `FillBestiarySpaceWithEntries` (`UIBestiaryEntryGrid.cs:37-71`): the
    // page start is the state's clamped `_atEntryIndex`; the fill walks
    // `min(_lastEntry, at + capacity)` over the WORKING SET.
    let hovered = grid_slot_at(screen, mouse);
    for slot in 0..capacity {
        let Some(idx) = ui.entry_at_slot(slot) else {
            break; // `num2 >= list.Count` (`:57-60`)
        };
        let entry = &crate::bestiary::ENTRIES[idx];
        let col = slot % cols;
        let row = slot / cols;
        let x = rect[0] + col as f32 * CELL;
        let y = rect[1] + row as f32 * CELL;
        let cell = [x, y, CELL, CELL];
        draw_card(r, lookups, cell, entry, tracker, hovered == Some(slot));
    }
    // `UIBestiaryEntryButton.DrawSelf` (`UIBestiaryEntryButton.cs:142-148`):
    // the hovered card prints the icon's hover text — the NPC name once
    // known, "???" while locked (`UnlockableNPCEntryIcon.GetHoverText`,
    // `:209-221`).
    if let Some(idx) = hovered.and_then(|s| ui.entry_at_slot(s)) {
        let entry = &crate::bestiary::ENTRIES[idx];
        let state = crate::bestiary::unlock_state(entry, tracker);
        super::tooltip::draw_mouse_text(
            r,
            lookups,
            mouse,
            &entry_hover_text(entry, state),
            screen,
            super::UiPulse::default(),
        );
    }
}

/// One `UIBestiaryEntryButton` (`:20-102`): the inner 68 px box holds
/// `Slot_Back` + the portrait (or the 15 % white `Icon_Locked`); the
/// `Slot_Overlay` (60 % white) / `Slot_Front` / `Slot_Selection` are
/// children of the OUTER 72 px element, centred (`HAlign/VAlign 0.5`,
/// `:71-91`) — the 70x70 sheets sit 1 px in. The border stack swaps on
/// hover (`MouseOver`/`MouseOut`, `:150-169`): idle overlay-then-front,
/// hovered front-then-glow (no overlay). The display index prints at
/// (10, 10) scale 0.9 (`:61-70`) — `NpcBestiarySortingId`, not the roster
/// position (`NPCNetIdBestiaryInfoElement.cs:10`).
fn draw_card(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    cell: [f32; 4],
    entry: &crate::bestiary::BestiaryEntryDef,
    tracker: &WorldBestiary,
    hover: bool,
) {
    let inner = inner_box(cell);
    let state = crate::bestiary::unlock_state(entry, tracker);
    // The inner container is `OverflowHidden = true`
    // (`UIBestiaryEntryButton.cs:30-38`): the back, the portrait and the
    // locked icon are its children and clip to the 68 px box; the overlay /
    // front / selection are appended to the *outer* element (`:84-91`) and
    // draw unclipped. `push_clip` is `UIElement.Draw`'s scissor enter
    // (`UIElement.cs:222-237`).
    r.push_clip(inner);
    if let Some(sheet) = lookups.ui_bestiary_slot_back {
        push_whole(r, lookups, &sheet, inner, [1.0; 4]);
    }
    if state.known() {
        draw_portrait(r, lookups, entry, inner, false);
    } else if let Some(locked) = lookups.ui_bestiary_locked {
        // `UIBestiaryEntryIcon.DrawSelf`'s locked branch (`:62-66`): the
        // texture at white * 0.15, drawn at `dimensions.Center()` with
        // origin = size/2 — natural size, centred, never stretched.
        let (lw, lh) = (locked.width as f32, locked.height as f32);
        let dst = [
            inner[0] + (inner[2] - lw) * 0.5,
            inner[1] + (inner[3] - lh) * 0.5,
            lw,
            lh,
        ];
        push_whole(r, lookups, &locked, dst, [0.15; 4]);
    }
    r.pop_clip();
    // The outer border layers: the 70x70 sheets centred in the 72 px cell
    // (`HAlign/VAlign 0.5` on the outer element). The layer ORDER is the
    // MouseOver/MouseOut swap (`entry_button_border_layers`).
    let centred = [cell[0] + 1.0, cell[1] + 1.0, cell[2] - 2.0, cell[3] - 2.0];
    let (draw_overlay, draw_glow) = entry_button_border_layers(hover, false);
    if draw_overlay {
        if let Some(overlay) = lookups.ui_bestiary_slot_overlay {
            push_whole(r, lookups, &overlay, centred, [0.6; 4]);
        }
    }
    if let Some(front) = lookups.ui_bestiary_slot_front {
        push_whole(r, lookups, &front, centred, [1.0; 4]);
    }
    if draw_glow {
        if let Some(sel) = lookups.ui_bestiary_slot_selection {
            push_whole(r, lookups, &sel, centred, [1.0; 4]);
        }
    }
    // The display index (`:58-70` — `TryGettingDisplayIndex` →
    // `NPCNetIdBestiaryInfoElement.BestiaryDisplayIndex`).
    text::hud_text(
        r,
        lookups,
        cell[0] + 10.0,
        cell[1] + 10.0,
        0.9,
        [1.0; 4],
        &format!("{}", crate::bestiary_filter::sorting_id(entry.net_id)),
    );
}

/// The entry's own bestiary NPC sheet, resolved through
/// `Lookups::bestiary_sheet_index` — the `BESTIARY_NPC_SHEET_IDS` order
/// table paired with `bestiary_npc_sheets`, keyed by the entry's vanilla
/// `net_id`. The literal `bestiary_npc_sheets[<slot>]` reads that used to
/// stand at every call site drifted as the table grew (the appended Old
/// One's Army rows shifted each one) and the Solar Selenian arm's `[174]`
/// ran one past the 174-entry table entirely, so the page panicked
/// (`BUG_REPORT.md` §15).
fn entry_sheet<'a>(
    lookups: &'a Lookups,
    entry: &crate::bestiary::BestiaryEntryDef,
) -> Option<&'a tr_render::SheetRect> {
    let slot = lookups.bestiary_sheet_index(entry.net_id as u16)?;
    lookups.bestiary_npc_sheets.get(slot)?.as_ref()
}

/// The entry button's inner container — `Width/Height (-4, 1)` centred in
/// the 72 px cell (`UIBestiaryEntryButton.cs:30-38`): the 68 px box, inset
/// 2 px per side. The `OverflowHidden` scissor rect and the icon box are
/// this same rect (`UIBestiaryEntryIcon.Update`'s `dimensions.ToRectangle()`).
fn inner_box(cell: [f32; 4]) -> [f32; 4] {
    [cell[0] + 2.0, cell[1] + 2.0, cell[2] - 4.0, cell[3] - 4.0]
}

/// The draw geometry of `UnlockableNPCEntryIcon.Update`/`UpdatePosition`
/// (`UnlockableNPCEntryIcon.cs:52-82`, `:112-123`) with the
/// `NPCID.Sets.NPCBestiaryDrawOffset` row applied: the NPC's position inside
/// the icon box, its scale and its draw frame. Pure so the tests can check
/// the per-NPC rows without a renderer.
///
/// `is_portrait` selects the `PortraitScale` /
/// `PortraitPositionX-YOverride` arms (`:56-59`, `:66-73`); the grid icon
/// passes `false` (`UIBestiaryEntryGrid.cs:61` — `isAPrettyPortrait: false`),
/// the info-page portrait `true` (`UIBestiaryNPCEntryPortrait.cs:56`).
fn portrait_layout(
    net_id: i32,
    floats: bool,
    frame_w: f32,
    frame_h: f32,
    iconbox: [f32; 4],
    is_portrait: bool,
) -> ([f32; 2], f32, usize) {
    let mut scale = 1.0;
    let mut offset = [0.0f32; 2];
    let mut frame_override: Option<i32> = None;
    if let Some(m) = super::bestiary_draw_offsets::get(net_id) {
        // `:54-64`: rotation/scale first, then the portrait-scale override.
        scale = m.scale;
        if is_portrait {
            if let Some(ps) = m.portrait_scale {
                scale = ps;
            }
        }
        offset = m.position;
        frame_override = m.frame;
        if is_portrait {
            if let Some(x) = m.portrait_position_x_override {
                offset[0] = x;
            }
            if let Some(y) = m.portrait_position_y_override {
                offset[1] = y;
            }
        }
    }
    // `UpdatePosition` (`:112-123`): `noGravity` NPCs centre on the box,
    // the rest bottom-anchor at `topLeft + size*(0.5, 1) + (0, -8)`; the
    // offset rides both, then `position.Floor()` (`:122`). The anchor uses
    // the NPC's unscaled `width`/`height` (`NPC.Bottom`/`Center` are
    // `position + width/height`, scale is draw-only).
    let (x, y) = if floats {
        (
            iconbox[0] + (iconbox[2] - frame_w) * 0.5 + offset[0],
            iconbox[1] + (iconbox[3] - frame_h) * 0.5 + offset[1],
        )
    } else {
        (
            iconbox[0] + iconbox[2] * 0.5 - frame_w * 0.5 + offset[0],
            iconbox[1] + iconbox[3] - frame_h - 8.0 + offset[1],
        )
    };
    // `:100-109`: without a `Frame` override the icon rests on the
    // `FindFrame` result (frame 0 at rest, velocity 0); with one,
    // `frame.Y = frame.Height * num`.
    let frame = frame_override.filter(|f| *f >= 0).unwrap_or(0) as usize;
    ([x.floor(), y.floor()], scale, frame)
}

/// `UnlockableNPCEntryIcon.Draw` (`:194-207`) — the NPC drawn through
/// `Main.DrawNPCDirect` at the icon box with the
/// `NPCID.Sets.NPCBestiaryDrawOffset` row's scale/position/frame
/// (`portrait_layout`); spill is clipped by the caller's `OverflowHidden`
/// scissor, never fit-scaled. The UI draws with full light; the slime
/// variants keep their `NPC.color` tint.
fn draw_portrait(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    entry: &crate::bestiary::BestiaryEntryDef,
    iconbox: [f32; 4],
    is_portrait: bool,
) {
    let mods = super::bestiary_draw_offsets::get(entry.net_id);
    // `sheet_i` is the derived slot `portrait_data` reports; the sheet
    // itself comes back through `entry_sheet` (the same derivation, the
    // bounds-checked form), so the index is not read again here.
    let (_sheet_i, frame_h, w, h, tint) = portrait_data(entry, lookups);
    // `UpdatePosition` (`UnlockableNPCEntryIcon.cs:112-123`) centres the icon
    // when `_npcCache.noGravity` is set, else bottom-anchors it. The cache runs
    // `SetDefaults` only, so the floater set is the generated `NO_GRAVITY`
    // table (NPC.cs SetDefaults type/aiStyle blocks) — not a hand-picked pair.
    let floats = super::bestiary_draw_offsets::no_gravity(entry.net_id);
                                                           // The NPC cache keeps its `SetDefaults` box even with a custom texture
                                                           // — `UpdatePosition` anchors on it, not on the portrait's size.
    let ([x, y], scale, frame) = portrait_layout(entry.net_id, floats, w, h, iconbox, is_portrait);
    // The custom single-frame portrait (`:74-81` load, `:197-201` draw):
    // drawn centred on the NPC position, rotation 0, origin = size/2,
    // `Color.White` — the NPC tint does not apply to it.
    if let Some(sheet) = mods
        .and_then(|m| m.custom_texture_path)
        .and_then(|_| lookups.bestiary_custom_portraits.get(&entry.net_id))
    {
        let (cw, ch) = (sheet.width as f32 * scale, sheet.height as f32 * scale);
        let dst = [x + w * 0.5 - cw * 0.5, y + h * 0.5 - ch * 0.5, cw, ch];
        push_whole(r, lookups, sheet, dst, [1.0; 4]);
        return;
    }
    let Some(sheet) = entry_sheet(lookups, entry) else {
        return;
    };
    let (dw, dh) = (w * scale, h * scale);
    // `DrawNPCDirect` draws at `npc.Center` with origin = frame/2 and
    // `npc.scale` (`Main.cs:22513+`): the scaled frame stays centred on the
    // NPC's unscaled box centre (`position + (width, height)/2`).
    let (x, y) = (x + (w - dw) * 0.5, y + (h - dh) * 0.5);
    // `DrawNPCDirect` draws the NPC's single `npc.frame` rect
    // (`UnlockableNPCEntryIcon.cs:194-207`), never the whole animation
    // sheet: `FindFrame` leaves `frame.Height` at one frame and the idle
    // icon rests on frame 0 (`Update`, `:100-109`). The sheet's row count
    // comes from the frame height `portrait_data` reports.
    let frames_y = portrait_frames_y(sheet.height, frame_h);
    let uv = sheet_frame_uv(&sheet, lookups.atlas_size, 1, frames_y, frame);
    let rotation = mods.map(|m| m.rotation).unwrap_or(0.0);
    if rotation == 0.0 {
        r.push_ui_sprite([x, y], [dw, dh], uv, tint);
    } else {
        // `DrawNPCDirect` draws at the NPC centre with origin = frame/2
        // (`Main.cs:22513+`, the `npc.rotation` arm): rotate about the
        // frame centre. `push_ui_sprite_rot` pivots the top-left, so shift
        // it by `origin − R(rot)·origin` (the `push_sprite_origin_rot`
        // form, `sprite.rs:786-801`).
        let origin = [dw * 0.5, dh * 0.5];
        let (s, c) = rotation.sin_cos();
        let tl = [
            x + origin[0] - (origin[0] * c - origin[1] * s),
            y + origin[1] - (origin[0] * s + origin[1] * c),
        ];
        r.push_ui_sprite_rot(tl, [dw, dh], uv, tint, rotation);
    }
}

/// The sheet's vertical frame count for a portrait: `sheet.height /
/// frame.Height` (`NPC.SetFrameSize`/`FindFrame`, used by
/// `UnlockableNPCEntryIcon.Update`, `UnlockableNPCEntryIcon.cs:100-109`).
/// At least one frame; a single-frame sheet stays whole.
fn portrait_frames_y(sheet_h: u32, frame_h: f32) -> usize {
    if frame_h > 0.0 {
        ((sheet_h as f32 / frame_h).round() as usize).max(1)
    } else {
        1
    }
}

/// The portrait parameters — the `NpcAssets` table's sizes
/// (`render/entities.rs::draw_npcs`) restated per entry; the slime frames
/// are 32x26 scaled by the variant's size scale and tinted with its
/// `NPC.color`.
#[allow(clippy::type_complexity)]
fn portrait_data(
    entry: &crate::bestiary::BestiaryEntryDef,
    lookups: &Lookups,
) -> (usize, f32, f32, f32, [f32; 4]) {
    // The sheet index rides `Lookups::bestiary_sheet_index` — the port's
    // order table `BESTIARY_NPC_SHEET_IDS` paired with the vec, derived
    // from the entry's own vanilla `net_id` (the hand-maintained table
    // that used to stand here had drifted twice: the frost trio's, the
    // boss rows' and finally the martian sheets' inserts shifted it).
    // Every sheet read below rides the same helper (`entry_sheet`): the
    // per-arm literal slots that survived here drifted the same way (the
    // Solar Selenian arm read `bestiary_npc_sheets[174]` — one past the
    // 174-entry table's last slot, so its page panicked).
    let sheet_i = lookups
        .bestiary_sheet_index(entry.net_id as u16)
        .unwrap_or(0);
    let white = [1.0, 1.0, 1.0, 1.0];
    match (entry.kind, entry.variant) {
        (NpcKind::Golem, _) => (sheet_i, 140.0, 140.0, 140.0, white),
        (NpcKind::GolemHead, _) => (sheet_i, 70.0, 70.0, 70.0, white),
        (NpcKind::GolemFistLeft, _) => (sheet_i, 30.0, 40.0, 30.0, white),
        (NpcKind::GolemFistRight, _) => (sheet_i, 30.0, 40.0, 30.0, white),
        (NpcKind::GolemHeadFree, _) => (sheet_i, 70.0, 70.0, 70.0, white),
        (NpcKind::Deerclops, _) => (sheet_i, 154.0, 60.0, 154.0, white),
        (NpcKind::SkeletronHand, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::WallOfFleshEye, _) => (sheet_i, 100.0, 100.0, 100.0, white),
        (NpcKind::PrimeCannon, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::PrimeSaw, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::PrimeVice, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::PrimeLaser, _) => (sheet_i, 52.0, 52.0, 52.0, white),
        (NpcKind::Slime, Some(v)) => {
            let s = v.stats();
            (
                sheet_i,
                26.0,
                32.0 * s.scale,
                26.0 * s.scale,
                [
                    s.color[0] as f32 / 255.0,
                    s.color[1] as f32 / 255.0,
                    s.color[2] as f32 / 255.0,
                    s.color[3] as f32 / 255.0,
                ],
            )
        }
        // The hungry (3 frames) / the leech segments (1).
        (NpcKind::Hungry, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 3.0)
                .unwrap_or(30.0);
            (sheet_i, h, 30.0, h, white)
        }
        (NpcKind::LeechHead, _) | (NpcKind::LeechBody, _) | (NpcKind::LeechTail, _) => {
            (sheet_i, 14.0, 14.0, 14.0, white)
        }
        // The eye: the 6-frame sheet's frame height rides the sheet
        // (the 100x110 frames).
        (NpcKind::EyeOfCthulhu, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(110.0);
            (sheet_i, h, 100.0, h, white)
        }
        (NpcKind::ServantOfCthulhu, _) => (sheet_i, 20.0, 20.0, 20.0, white),
        (NpcKind::Slime, None) => (sheet_i, 26.0, 32.0, 26.0, white),
        (NpcKind::DemonEye, _) => (sheet_i, 24.0, 38.0, 24.0, white),
        (NpcKind::WanderingEye, _) => {
            // The 4-frame sheet's frame height rides the sheet itself
            // (`draw_npcs`'s `t.h / 4`).
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(24.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::Zombie, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Guide, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Skeleton | NpcKind::UndeadMiner, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::CaveBat, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::JungleBat, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::Hellbat, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::VoodooDemon, _) => (sheet_i, 18.0, 26.0, 18.0, white),
        (NpcKind::EaterOfSouls, _) => (sheet_i, 30.0, 30.0, 30.0, white),
        (NpcKind::Hornet, _) => (sheet_i, 32.0, 34.0, 32.0, white),
        // The volley bees (S1-D-3 W-3): the 4-row strips
        // (`Main.npcFrameCount[210/211] = 4`, `Main.cs:67082`); the frame
        // height rides the sheet itself (`draw_npcs`'s generic anchor).
        (NpcKind::Bee | NpcKind::BeeSmall, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(12.0);
            (sheet_i, h, h, h, white)
        }
        (NpcKind::MeteorHead, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        // The casters ride the 38x48 humanoid frame box; the spheres the
        // 16x16 sheet.
        (NpcKind::GoblinSorcerer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Tim, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::DarkCaster, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::FireImp, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        // The Desert Djinn: the 16-frame NPC_533 strip's own row height.
        (NpcKind::DesertDjinn, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(62.0);
            (sheet_i, h, 30.0, h, white)
        }
        (NpcKind::ChaosBall, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::BurningSphere, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::WaterSphere, _) => (sheet_i, 16.0, 16.0, 16.0, white),
        (NpcKind::Goldfish, _) => (sheet_i, 18.0, 20.0, 18.0, white),
        (NpcKind::Piranha, _) => (sheet_i, 20.0, 18.0, 20.0, white),
        (NpcKind::Shark, _) => (sheet_i, 24.0, 100.0, 24.0, white),
        // The worm segments: the square sheets.
        (NpcKind::DevourerHead, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::DevourerBody, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::DevourerTail, _) => (sheet_i, 22.0, 22.0, 22.0, white),
        (NpcKind::GiantWormHead, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::GiantWormBody, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::GiantWormTail, _) => (sheet_i, 14.0, 14.0, 14.0, white),
        (NpcKind::EaterOfWorldsHead, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        (NpcKind::EaterOfWorldsBody, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        (NpcKind::EaterOfWorldsTail, _) => (sheet_i, 38.0, 38.0, 38.0, white),
        // The wyvern head's portrait: the 32x32 single-frame box
        // (`NPC.cs:9943-9944`, `Main.npcFrameCount[87]` = 1). The sheet
        // itself is deferred (the ~71-px atlas headroom, ANALYSIS "The
        // Wyvern family") — `entry_sheet` returns None and the portrait
        // draw skips until the repack; the vanilla's draw modifiers
        // (`NPCID.cs:1811-1819`: the NPC_87 custom path, Position
        // (55, 15), the portrait overrides (4, 10)) ride that wave too.
        // The segments never reach this arm through the catalog (the
        // head's fold, `NPCID.cs:1821-1856`'s `Hide = true`) — the rows
        // exist for the match's exhaustiveness, the same 32x32 box.
        (NpcKind::WyvernHead, _) => (sheet_i, 32.0, 32.0, 32.0, white),
        (NpcKind::WyvernLegs, _)
        | (NpcKind::WyvernBody, _)
        | (NpcKind::WyvernBody2, _)
        | (NpcKind::WyvernBody3, _)
        | (NpcKind::WyvernTail, _) => (sheet_i, 32.0, 32.0, 32.0, white),
        (NpcKind::Merchant, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Dryad, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Nurse, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::ArmsDealer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Demolitionist, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Painter, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::WitchDoctor, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Clothier, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::TravellingMerchant, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Angler, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::SleepingAngler, _) => (sheet_i, 8.0, 34.0, 8.0, white),
        (NpcKind::Wizard, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Mechanic, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::BoundWizard, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::BoundMechanic, _) => (sheet_i, 40.0, 38.0, 40.0, white),
        (NpcKind::Stylist, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::TaxCollector, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Golfer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Zoologist, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::Princess, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::DyeTrader, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::PartyGirl, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::SleepingGolfer, _) => (sheet_i, 40.0, 38.0, 40.0, white),
        (NpcKind::WebbedStylist, _) => (sheet_i, 40.0, 38.0, 40.0, white),
        (NpcKind::EyeOfCthulhu, _) => (sheet_i, 110.0, 100.0, 110.0, white),
        (NpcKind::QueenBee, _) => (sheet_i, 66.0, 66.0, 66.0, white),
        (NpcKind::SkeletronHead, _) => (sheet_i, 102.0, 80.0, 102.0, white),
        (NpcKind::WallOfFlesh, _) => (sheet_i, 120.0, 120.0, 120.0, white),
        (NpcKind::TheDestroyer, _) => (sheet_i, 48.0, 48.0, 48.0, white),
        // The segments fold onto the head's entry (never drawn alone);
        // the arms keep the match exhaustive at the SetDefaults boxes.
        (NpcKind::TheDestroyerBody | NpcKind::TheDestroyerTail, _) => {
            (sheet_i, 48.0, 48.0, 48.0, white)
        }
        (NpcKind::Probe, _) => (sheet_i, 30.0, 30.0, 30.0, white),
        // S1-D-3 W-9 fishron: the boss + the Sharkron have their own
        // bestiary entries; the Hide=true hazards fold onto the boss
        // (never drawn alone) — the arms keep the match exhaustive at
        // the SetDefaults boxes (the destroyer-segment precedent).
        (NpcKind::DukeFishron, _) => (sheet_i, 150.0, 100.0, 150.0, white),
        (NpcKind::DetonatingBubble, _) => (sheet_i, 36.0, 36.0, 36.0, white),
        (NpcKind::Sharkron, _) => (sheet_i, 120.0, 24.0, 120.0, white),
        (NpcKind::Sharkron2, _) => (sheet_i, 100.0, 24.0, 100.0, white),
        (NpcKind::SkeletronPrime, _) => (sheet_i, 102.0, 80.0, 102.0, white),
        (NpcKind::Retinazer, _) => (sheet_i, 110.0, 100.0, 110.0, white),
        (NpcKind::Spazmatism, _) => (sheet_i, 110.0, 100.0, 110.0, white),
        (NpcKind::Plantera, _) => (sheet_i, 86.0, 86.0, 86.0, white),
        // The Plantera minions (S1-D-3-wire-plantera-minions): the
        // portrait rests each kind's `SetDefaults` box
        // (`NPC.cs:12364-12437`) - the pumpkin/frost arms' shape.
        (NpcKind::FungiSpore, _) => (sheet_i, 12.0, 12.0, 12.0, white),
        (NpcKind::PlanterasHook, _) => (sheet_i, 40.0, 40.0, 40.0, white),
        (NpcKind::PlanterasTentacle, _) => (sheet_i, 24.0, 24.0, 24.0, white),
        (NpcKind::Spore, _) => (sheet_i, 18.0, 18.0, 18.0, white),
        (NpcKind::BrainOfCthulhu, _) => (sheet_i, 110.0, 160.0, 110.0, white),
        (NpcKind::Creeper, _) => (sheet_i, 30.0, 30.0, 30.0, white),
        (NpcKind::GoblinPeon, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::GoblinThief, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::GoblinWarrior, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::GoblinArcher, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        (NpcKind::BoundGoblin, _) => (sheet_i, 48.0, 34.0, 54.0, white),
        (NpcKind::GoblinTinkerer, _) => (sheet_i, 48.0, 38.0, 48.0, white),
        // The frost trio: the 4/5/7-frame strips over the 26x40 frames.
        (NpcKind::SnowmanGangsta, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(40.0);
            (sheet_i, h, 26.0, h, white)
        }
        (NpcKind::SnowBalla, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(40.0);
            (sheet_i, h, 26.0, h, white)
        }
        (NpcKind::MysterySnowman, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 7.0)
                .unwrap_or(40.0);
            (sheet_i, h, 26.0, h, white)
        }
        // The martians: the 9-frame strips over the 18x40 frames (the
        // walker's 11-frame 34x110 sheet).
        (NpcKind::MartianOfficer, _)
        | (NpcKind::GrayGrunt, _)
        | (NpcKind::MartianEngineer, _)
        | (NpcKind::GigaZapper, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::MartianWalker, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(110.0);
            (sheet_i, h, 34.0, h, white)
        }
        // The probe: the 8-frame NPC_399 strip over the 42x72 frames.
        (NpcKind::MartianProbe, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(72.0);
            (sheet_i, h, 42.0, h, white)
        }
        // The drone: the 2-frame NPC_388 strip over the 40x30 frames.
        (NpcKind::MartianDrone, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 2.0)
                .unwrap_or(30.0);
            (sheet_i, h, 40.0, h, white)
        }
        // The Tesla Turret (S1-F-13d): the 2-frame NPC_387 strip over the
        // 20x50 frames.
        (NpcKind::MartianTurret, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 2.0)
                .unwrap_or(50.0);
            (sheet_i, h, 20.0, h, white)
        }
        // The bubble: hidden in the bestiary (`NPCID.cs:3766-3770`'s
        // `Hide = true`) — it folds onto the Officer's entry, so this arm
        // never renders; the SetDefaults box (18x40, `NPC.cs:13885`/
        // `:13887`) stands as the shape.
        (NpcKind::ForceBubble, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        // The shooters: the 9-frame NPC_381/382 strips over the 18x40
        // frames.
        (NpcKind::BrainScrambler | NpcKind::RayGunner, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The rider: the 5-frame NPC_390 strip over the 18x40 frames.
        (NpcKind::ScutlixRider, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The carrier: the 6-frame NPC_391 strip over the 60x40 frames.
        (NpcKind::Scutlix, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(40.0);
            (sheet_i, h, 60.0, h, white)
        }
        // The saucer family: the shell 4-frame, the turret 18-frame, the
        // cannon 8-frame, the core 11-frame strips.
        (NpcKind::MartianSaucer, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(80.0);
            (sheet_i, h, 150.0, h, white)
        }
        (NpcKind::MartianSaucerTurret, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 18.0)
                .unwrap_or(16.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::MartianSaucerCannon, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(36.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::MartianSaucerCore, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 11.0)
                .unwrap_or(90.0);
            (sheet_i, h, 120.0, h, white)
        }
        // The Dutchman: the 1-frame ship strip + the cannons' 9-frame.
        (NpcKind::FlyingDutchman, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32)
                .unwrap_or(120.0);
            (sheet_i, h, 350.0, h, white)
        }
        (NpcKind::FlyingDutchmanCannon, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(30.0);
            (sheet_i, h, 30.0, h, white)
        }
        // The DD2 goblins: the 14-frame NPC_552-554 strips over the 18x40
        // frames.
        (NpcKind::DD2GoblinT1 | NpcKind::DD2GoblinT2 | NpcKind::DD2GoblinT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The support family: the bombers'/skeletons' 18x40 and the
        // javelin throwers' 26x52 strips.
        (
            NpcKind::DD2GoblinBomberT1
            | NpcKind::DD2GoblinBomberT2
            | NpcKind::DD2GoblinBomberT3
            | NpcKind::DD2SkeletonT1
            | NpcKind::DD2SkeletonT3,
            _,
        ) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::DD2JavelinT1 | NpcKind::DD2JavelinT2 | NpcKind::DD2JavelinT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(52.0);
            (sheet_i, h, 26.0, h, white)
        }
        // The Dark Mage: the 20-frame NPC_564 strip over the 34x62 frames.
        (NpcKind::DD2DarkMage, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 20.0)
                .unwrap_or(62.0);
            (sheet_i, h, 34.0, h, white)
        }
        // Betsy: the 11-frame NPC_551 strip over the 190x90 frames.
        (NpcKind::DD2Betsy, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 11.0)
                .unwrap_or(90.0);
            (sheet_i, h, 190.0, h, white)
        }
        // The WitherBeast: the 14-frame NPC_568 strip over the 18x40 frames.
        (NpcKind::DD2WitherBeast, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 14.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The tavernkeeper: the 16-frame NPC_550 strip over the 18x40.
        (NpcKind::DD2Bartender, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        // The crystal: the NPC_548 strip over the 40x40 frames.
        (NpcKind::DD2EterniaCrystal, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32)
                .unwrap_or(40.0);
            (sheet_i, h, 40.0, h, white)
        }
        // The cultist: the 1-frame NPC_422 strip over the 130x270 frames.
        (NpcKind::AncientCultist, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32)
                .unwrap_or(270.0);
            (sheet_i, h, 130.0, h, white)
        }
        // The towers: the 1-frame NPC_493/507/517 strips.
        // The four pillars: the single 174x364 frames (the sheet's own
        // height — the hand-held 270 read of `bestiary_npc_sheets[111]`
        // was the Stardust row for every kind).
        (
            NpcKind::StardustTower
            | NpcKind::NebulaTower
            | NpcKind::SolarTower
            | NpcKind::VortexTower,
            _,
        ) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32)
                .unwrap_or(364.0);
            (sheet_i, h, 174.0, h, white)
        }
        // The Old One's Army's lane gate (M4-EVENT-18): the single 78x130
        // NPC_549 frame — the body's own size (`NPC.cs`'s case 549).
        (NpcKind::DD2LanePortal, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32)
                .unwrap_or(130.0);
            (sheet_i, h, 78.0, h, white)
        }
        // The Old One's Army's tier-2/3 families (M4-EVENT-19):
        // their own sheets and `Main.npcFrameCount` rows.
        (NpcKind::DD2WyvernT1, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(38.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::DD2WyvernT2, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(38.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::DD2WyvernT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(38.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::DD2DarkMageT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 20.0)
                .unwrap_or(62.0);
            (sheet_i, h, 34.0, h, white)
        }
        (NpcKind::DD2WitherBeastT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 17.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::DD2DrakinT2, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(58.0);
            (sheet_i, h, 42.0, h, white)
        }
        (NpcKind::DD2DrakinT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(58.0);
            (sheet_i, h, 42.0, h, white)
        }
        (NpcKind::DD2KoboldWalkerT2, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 18.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::DD2KoboldWalkerT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 18.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::DD2KoboldFlyerT2, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(38.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::DD2KoboldFlyerT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(38.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::DD2OgreT2, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(124.0);
            (sheet_i, h, 96.0, h, white)
        }
        (NpcKind::DD2OgreT3, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(124.0);
            (sheet_i, h, 96.0, h, white)
        }
        // The shards' portraits.
        (NpcKind::StardustCellBig, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(44.0);
            (sheet_i, h, 44.0, h, white)
        }
        (NpcKind::StardustCellSmall, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 2.0)
                .unwrap_or(22.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::StardustJellyfishBig, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(60.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::NebulaBeast, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 12.0)
                .unwrap_or(44.0);
            (sheet_i, h, 50.0, h, white)
        }
        (NpcKind::VortexRifleman | NpcKind::VortexSoldier, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 13.0)
                .unwrap_or(56.0);
            (sheet_i, h, 30.0, h, white)
        }
        // The pirates: the 16/20-frame strips over the 18x40 frames.
        // The Moon Lord family's portraits over the 396 sheet's rows.
        // The vortex trio's portraits over the 426 sheet's 15 rows.
        // The nebula trio's portraits over the 420 sheet's rows.
        // The tower mobs' portraits over the 409 sheet's rows.
        (NpcKind::StardustSpiderBig, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(42.0);
            (sheet_i, h, 34.0, h, white)
        }
        (NpcKind::StardustSpiderSmall, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 11.0)
                .unwrap_or(22.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::StardustSoldier, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::SolarDrakomire, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(40.0);
            (sheet_i, h, 60.0, h, white)
        }
        (NpcKind::SolarDrakomireRider, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 13.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::SolarSroller, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(52.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::SolarCorite, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 15.0)
                .unwrap_or(38.0);
            (sheet_i, h, 38.0, h, white)
        }
        // The pillar worms' portraits over the 412 sheet's rows.
        (NpcKind::SolarCrawltipedeHead, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(20.0);
            (sheet_i, h, 20.0, h, white)
        }
        (NpcKind::SolarCrawltipedeBody, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(20.0);
            (sheet_i, h, 20.0, h, white)
        }
        (NpcKind::SolarCrawltipedeTail, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(20.0);
            (sheet_i, h, 20.0, h, white)
        }
        (NpcKind::StardustWormHead, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 1.0)
                .unwrap_or(32.0);
            (sheet_i, h, 32.0, h, white)
        }
        (NpcKind::SolarSpearman, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 10.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        // The Selenian (`NPCID.SolarSolenian`, `NPCID.cs:11515` — net id 419,
        // the bestiary entry's own `net_id`): the 8-row NPC_419 strip, so the
        // frame height rides its own sheet through `entry_sheet`. The literal
        // `bestiary_npc_sheets[174]` that stood here was one past the table's
        // last slot (174 entries, 0..=173) and panicked the page (§15).
        (NpcKind::SolarSelenian, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::NebulaFloater, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(40.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::BrainSuckler, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 6.0)
                .unwrap_or(44.0);
            (sheet_i, h, 44.0, h, white)
        }
        (NpcKind::Predictor, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 12.0)
                .unwrap_or(56.0);
            (sheet_i, h, 22.0, h, white)
        }
        (NpcKind::VortexHunter, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 15.0)
                .unwrap_or(62.0);
            (sheet_i, h, 50.0, h, white)
        }
        (NpcKind::VortexAlien, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 9.0)
                .unwrap_or(28.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::VortexHeadcrab, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 5.0)
                .unwrap_or(26.0);
            (sheet_i, h, 24.0, h, white)
        }
        (NpcKind::MoonLordHead, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(56.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::MoonLordHand, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(66.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::MoonLordCore, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(66.0);
            (sheet_i, h, 46.0, h, white)
        }
        (NpcKind::MoonLordFreeEye, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(60.0);
            (sheet_i, h, 60.0, h, white)
        }
        (NpcKind::MoonLordLeechBlob, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 8.0)
                .unwrap_or(30.0);
            (sheet_i, h, 30.0, h, white)
        }
        (NpcKind::PirateDeckhand, _)
        | (NpcKind::PirateCaptainsMate, _)
        | (NpcKind::PirateArcher, _)
        | (NpcKind::PirateCorsair, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::PirateCaptain, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 20.0)
                .unwrap_or(40.0);
            (sheet_i, h, 18.0, h, white)
        }
        (NpcKind::KingSlime, _) => (sheet_i, 120.0, 174.0, 120.0, white),
        // The goblins: the 16/21/20-frame humanoid strips over the 38-wide
        // display boxes (the frame height rides each sheet).
        (NpcKind::GoblinThief, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinWarrior, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinArcher, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 16.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinPeon, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 21.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::GoblinSummoner, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 20.0)
                .unwrap_or(48.0);
            (sheet_i, h, 38.0, h, white)
        }
        // The Pumpkin Moon family (`S1-F-13b`): the kinds carry no
        // bestiary entry yet (Phase 2's render registration lands the
        // sheets and the entries), so the portrait rests the kind's
        // `SetDefaults` box — the shape the C#'s `NpcAssets` rows carry
        // (`NPC.cs:12929-13203`).
        (NpcKind::Scarecrow1, _)
        | (NpcKind::Scarecrow2, _)
        | (NpcKind::Scarecrow3, _)
        | (NpcKind::Scarecrow4, _)
        | (NpcKind::Scarecrow5, _)
        | (NpcKind::Scarecrow6, _)
        | (NpcKind::Scarecrow7, _)
        | (NpcKind::Scarecrow8, _)
        | (NpcKind::Scarecrow9, _)
        | (NpcKind::Scarecrow10, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::Splinterling, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::HeadlessHorseman, _) => (sheet_i, 70.0, 74.0, 70.0, white),
        (NpcKind::MourningWood, _) => (sheet_i, 154.0, 164.0, 154.0, white),
        (NpcKind::Pumpking, _) => (sheet_i, 100.0, 100.0, 100.0, white),
        (NpcKind::Hellhound, _) => (sheet_i, 30.0, 46.0, 30.0, white),
        (NpcKind::Poltergeist, _) => (sheet_i, 44.0, 24.0, 44.0, white),
        // The Frost Moon wave family (`S1-F-13c` slice B): the portrait
        // rests each kind's `SetDefaults` box (`NPC.cs:13369-13487`) — the
        // same shape the pumpkin rows above carry.
        (NpcKind::IceQueen, _) => (sheet_i, 140.0, 130.0, 140.0, white),
        (NpcKind::SantaNK1, _) => (sheet_i, 140.0, 112.0, 140.0, white),
        (NpcKind::ElfCopter, _) => (sheet_i, 50.0, 50.0, 50.0, white),
        (NpcKind::Nutcracker, _) | (NpcKind::NutcrackerSpinning, _) => {
            (sheet_i, 76.0, 28.0, 76.0, white)
        }
        (NpcKind::ElfArcher, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::Krampus, _) => (sheet_i, 90.0, 18.0, 90.0, white),
        (NpcKind::Flocko, _) => (sheet_i, 54.0, 54.0, 54.0, white),
        // The Frost Moon family, slice A (`S1-F-13c`): the same shape as
        // the pumpkin arms above — the portrait rests the kind's
        // `SetDefaults` box (`NPC.cs:13281-13368`): the trio + the
        // Gingerbread Man the 18x40 humanoid box, the Present Mimic the
        // 24x24 chest box, the Yeti the 38x78 box and the Everscream the
        // 172x130 box.
        (NpcKind::ZombieElf, _)
        | (NpcKind::ZombieElfBeard, _)
        | (NpcKind::ZombieElfGirl, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::PresentMimic, _) => (sheet_i, 24.0, 24.0, 24.0, white),
        (NpcKind::GingerbreadMan, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::Yeti, _) => (sheet_i, 78.0, 38.0, 78.0, white),
        (NpcKind::Everscream, _) => (sheet_i, 130.0, 172.0, 130.0, white),
        // S1-D-3ff wave A: the portrait rests the kind's `SetDefaults` box
        // (`npc/data.rs::size`, the C# rows cited there) — the pumpkin/frost
        // arms' shape.
        (NpcKind::MotherSlime | NpcKind::DungeonSlime, _) => (sheet_i, 24.0, 36.0, 24.0, white),
        (NpcKind::LavaSlime | NpcKind::IlluminantSlime | NpcKind::IceSlime | NpcKind::SpikedIceSlime, _) => {
            (sheet_i, 18.0, 24.0, 18.0, white)
        }
        (NpcKind::CorruptSlime | NpcKind::Crimslime, _) => (sheet_i, 30.0, 40.0, 30.0, white),
        (NpcKind::UmbrellaSlime, _) => (sheet_i, 26.0, 38.0, 26.0, white),
        (NpcKind::CataractEye
        | NpcKind::SleepyEye
        | NpcKind::DialatedEye
        | NpcKind::GreenEye
        | NpcKind::PurpleEye, _) => (sheet_i, 32.0, 30.0, 32.0, white),
        (NpcKind::DoctorBones
        | NpcKind::TheGroom
        | NpcKind::BaldZombie
        | NpcKind::ZombieEskimo
        | NpcKind::PincushionZombie
        | NpcKind::SlimedZombie
        | NpcKind::SwampZombie
        | NpcKind::TwiggyZombie
        | NpcKind::FemaleZombie
        | NpcKind::ZombieRaincoat, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        // The small-kinds wiring wave (S1-D-3-wire-smallkinds): the four
        // wired kinds' portraits — the sheet's own row height over
        // `Main.npcFrameCount[type]` (72/250 four frames, 288 three, 384
        // one; `Main.cs:67082`), the frame width the sheet's own.
        (NpcKind::BlazingWheel, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(40.0);
            (sheet_i, h, 40.0, h, white)
        }
        (NpcKind::AngryNimbus, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 4.0)
                .unwrap_or(30.0);
            (sheet_i, h, 38.0, h, white)
        }
        (NpcKind::DungeonSpirit, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32 / 3.0)
                .unwrap_or(26.0);
            (sheet_i, h, 26.0, h, white)
        }
        (NpcKind::ForceBubble, _) => {
            let h = entry_sheet(lookups, entry)
                .map(|s| s.height as f32)
                .unwrap_or(46.0);
            (sheet_i, h, 22.0, h, white)
        }
        // S1-D-3z — the holiday zombies' portrait rests the same 18x40
        // `SetDefaults` box (`NPC.cs:13035-13036`/`:13206-13207`).
        (NpcKind::ZombieDoctor
        | NpcKind::ZombieSuperman
        | NpcKind::ZombiePixie
        | NpcKind::ZombieXmas
        | NpcKind::ZombieSweater, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        // S1-D-3z — the blood-moon pair's portrait rests the kind's
        // `SetDefaults` box: 489 the 18x40 humanoid box (`NPC.cs:14607-
        // 14608`), 490 the 28x30 flyer box (`:14621-14622`).
        (NpcKind::BloodZombie, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::Drippler, _) => (sheet_i, 30.0, 28.0, 30.0, white),
    }
}

/// One info-page row (`AddInfoToList`'s `_list` items,
/// `UIBestiaryEntryInfoPage.cs:128-163`): the category-ordered elements
/// plus the `UIHorizontalSeparator` between groups. The port keeps the
/// pre-existing row conventions for the nameplate/portrait/flavor/drop
/// rows (documented); the stats block is the C#'s 2x2 grid + coin row.
enum InfoRow {
    /// `NamePlateInfoElement` — a 24 px row (`NamePlateInfoElement.cs:28`).
    Nameplate,
    /// `NPCPortraitInfoElement` — the 112 px card row (`:26`).
    Portrait,
    /// `FlavorTextBestiaryInfoElement` — the wrapped panel, height measured.
    Flavor(String),
    /// `NPCStatsReportInfoElement` — the 2x2 grid + coin row.
    Stats,
    /// `NPCKillCounterInfoElement` — the 30 px panel row (`:84`).
    KillCounter(String),
    /// `ItemDropBestiaryInfoElement` — one `UIBestiaryInfoItemLine`.
    Drop { name: String, rate: String, locked: bool },
    /// `UIHorizontalSeparator` (`:153-158`).
    Separator,
}

/// The separator row's height — the port's 2 px convention (the C# uses
/// the `Separator1` sheet's 4 px; the info page keeps the pre-existing
/// flat-quad separator, a documented row convention).
const INFO_SEPARATOR_H: f32 = 2.0;

/// The flavor panel's wrapped height (`FlavorTextBestiaryInfoElement`'s
/// `AddDynamicResize`, `:47-53`): the port's wrapped-lines convention —
/// `lines * line_h + 8` (the C# adds `WrappedTextBottomPadding 20 * scale`
/// instead; the port's tighter panel is the pre-existing convention).
fn flavor_row_height(r: &SpriteRenderer, lookups: &Lookups, flavor: &str, panel_w: f32) -> f32 {
    let lines = wrap_text(r, lookups, flavor, 0.8, panel_w - 8.0);
    lines.len() as f32 * (text::line_height(lookups) * 0.8) + 8.0
}

/// `AddInfoToList`'s row build (`UIBestiaryEntryInfoPage.cs:128-163`):
/// the category order Nameplate → Portrait → FlavorText → Stats →
/// ItemsFromDrops, a separator after every non-empty group except the
/// last (`:151-162`). The stats row's height is resolved here (the coin
/// gate) and carried alongside.
fn build_info_rows(
    r: &SpriteRenderer,
    lookups: &Lookups,
    entry: &crate::bestiary::BestiaryEntryDef,
    state: UnlockState,
    tracker: &WorldBestiary,
    content_w: f32,
) -> Vec<(InfoRow, f32)> {
    let mut groups: Vec<Vec<(InfoRow, f32)>> = Vec::new();
    // 1. Nameplate — always produced (`NamePlateInfoElement.cs:19-31`).
    groups.push(vec![(InfoRow::Nameplate, 24.0)]);
    // 2. Portrait — always produced (`NPCPortraitInfoElement.cs:21-95`).
    groups.push(vec![(InfoRow::Portrait, NPC_PORTRAIT_SIZE[1])]);
    // 3. FlavorText — tier 2+ and the language row exists.
    if let Some(flavor) = flavor_text(entry, state) {
        let h = flavor_row_height(r, lookups, &flavor, content_w - 11.0);
        groups.push(vec![(InfoRow::Flavor(flavor), h)]);
    }
    // 4. Stats — the block from tier 1, the kill counter when kills >= 1.
    if state.known() {
        let stats =
            crate::npc::entry_stats(entry.kind, entry.variant.unwrap_or(SlimeVariant::Blue));
        let mut g = vec![(InfoRow::Stats, stats_block_height(stats.value > 0.0))];
        let (kills, _needed) = crate::bestiary::kill_progress(entry, &tracker.kills);
        if let Some(count) = kill_counter_text(kills) {
            g.push((InfoRow::KillCounter(count), 30.0));
        }
        groups.push(g);
    }
    // 5. ItemsFromDrops — tier 2+ (`ItemDropBestiaryInfoElement.cs:20-23`).
    if state >= UnlockState::CanShowStats {
        let rows = crate::loot_db::drop_rows(entry.net_id);
        let mut g = Vec::new();
        for row in &rows {
            let name = crate::registry::name(row.item_id as u16);
            let rate = drop_rate_text(state, row.drop_rate, row.stack_min, row.stack_max);
            let locked = state < UnlockState::CanShowDropsWithoutDropRates;
            let h = if locked { 32.0 } else { 22.0 };
            g.push((InfoRow::Drop { name, rate, locked }, h));
        }
        if !g.is_empty() {
            groups.push(g);
        }
    }
    // Interleave the separators between groups, none after the last.
    let mut out = Vec::new();
    let n = groups.len();
    for (i, g) in groups.into_iter().enumerate() {
        out.extend(g);
        if i + 1 < n {
            out.push((InfoRow::Separator, INFO_SEPARATOR_H));
        }
    }
    out
}

/// The info page's total content height — `UIList.GetTotalHeight`
/// (`UIList.cs:63-66`) over the built rows, the scrollbar's `maxViewSize`.
/// The wheel handler and the draw share this so the clamp matches.
pub fn info_page_content_height(
    r: &SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &WorldBestiary,
    selected: Option<usize>,
) -> f32 {
    let Some(idx) = selected else {
        return 0.0; // `FillInfoForEntry(null)` — the empty list.
    };
    let page = info_page_rect(&panel_rect(screen));
    let content_w = page[2] - 4.0 - 20.0;
    let entry = &crate::bestiary::ENTRIES[idx];
    let state = crate::bestiary::unlock_state(entry, tracker);
    let rows = build_info_rows(r, lookups, entry, state, tracker, content_w);
    let heights: Vec<f32> = rows.iter().map(|(_, h)| *h).collect();
    info_page_row_layout(&heights, INFO_PAGE_LIST_PADDING[4]).1
}

/// The info page (`UIBestiaryEntryInfoPage`) — the 230 px panel beside the
/// grid (`info_page_rect`), border `(89, 116, 213)` / bg `(73, 94, 171)`
/// (`:33-34`). `FillInfoForEntry(null)` empties the list (`:111-119`), so
/// `None` draws just the panel chrome. The rows follow `AddInfoToList`'s
/// category order (`:128-163`): Nameplate → Portrait → FlavorText → Stats →
/// ItemsFromDrops, a separator between groups (`:151-160`). The list's
/// `(2, 4, 2, 4)` padding + `ListPadding 4` (`:40-45`) and the always-
/// attached scrollbar's `-20` width (`CheckScrollBar:90-109`, the forced
/// `canScroll = true`) set the content box. `scroll` is the scrollbar's
/// `ViewPosition` — the rows ride `-_viewPosition` (`UIList.DrawSelf:181`)
/// inside the list's `OverflowHidden` clip, and the thumb draws over the
/// reserved 20 px column (W4-e23 remainder, the g53 scrollbar pattern).
fn draw_info_page(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &WorldBestiary,
    selected: Option<usize>,
    mouse: Vec2,
    scroll: f32,
) {
    let page = info_page_rect(&panel_rect(screen));
    // The panel body + border (`UIPanel.DrawSelf`, the ctor colours).
    let (border, bg) = info_page_colors();
    r.push_ui_sprite([page[0], page[1]], [page[2], page[3]], r.white_uv(), super::color::tint(bg));
    super::panel::draw_panel(r, lookups, lookups.ui_panel_border, page, super::color::tint(border));
    let Some(idx) = selected else {
        return; // `FillInfoForEntry(null)` — the empty list.
    };
    let entry = &crate::bestiary::ENTRIES[idx];
    let state = crate::bestiary::unlock_state(entry, tracker);
    // The list content box: padding (2, 4, 2, 4) + the scrollbar's -20 width.
    let content = [page[0] + 2.0, page[1] + 4.0, page[2] - 4.0 - 20.0, page[3] - 8.0];
    let rows = build_info_rows(r, lookups, entry, state, tracker, content[2]);
    let heights: Vec<f32> = rows.iter().map(|(_, h)| *h).collect();
    let (tops, total_h) = info_page_row_layout(&heights, INFO_PAGE_LIST_PADDING[4]);
    let view_h = info_page_view_height(page);
    let (view, pos) = info_page_set_view(view_h, total_h, scroll);
    // The list's `OverflowHidden` scissor (`UIList.cs:59`) — the rows clip
    // to the content box while they ride `-_viewPosition`.
    r.push_clip(content);
    for ((row, _), &top) in rows.iter().zip(tops.iter()) {
        let ty = content[1] + top - pos;
        draw_info_row(r, lookups, row, content, ty, entry, state, tracker, screen, mouse);
    }
    r.pop_clip();
    // The scrollbar thumb (`UIScrollbar.DrawSelf`, `UIScrollbar.cs:103-124`)
    // over the reserved column — the shared g53 pattern. The bestiary's
    // `UIScrollbar` is the default Blue theme (`UIBestiaryEntryInfoPage.cs:47`
    // — `new UIScrollbar()`), so the track is `Images/UI/Scrollbar`.
    let bar = info_page_scrollbar_rect(page, 20.0);
    super::panel::draw_scrollbar_themed(r, lookups, bar, view, total_h, pos, false, lookups.ui_scrollbar_blue);
}

/// One info-page row's draw at its scrolled top `ty`.
fn draw_info_row(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    row: &InfoRow,
    content: [f32; 4],
    ty: f32,
    entry: &crate::bestiary::BestiaryEntryDef,
    state: UnlockState,
    tracker: &WorldBestiary,
    screen: Vec2,
    mouse: Vec2,
) {
    let tx = content[0];
    match row {
        InfoRow::Nameplate => {
            // `NamePlateInfoElement.cs:19-31`: the name centred
            // (`HAlign/VAlign 0.5`, `Top 2`), "???" below tier 1.
            let shown_name = entry_hover_text(entry, state);
            let nw = text::hud_text_width(r, lookups, &shown_name, 1.0);
            text::hud_text(
                r,
                lookups,
                content[0] + (content[2] - nw) * 0.5,
                ty + 2.0 + (24.0 - text::line_height(lookups)) * 0.5,
                1.0,
                [1.0; 4],
                &shown_name,
            );
        }
        InfoRow::Portrait => {
            // `NPCPortraitInfoElement.cs:21-95`: the 193x112 card at
            // `Left 4` (`:87`), the icon `isPortrait: true`.
            let card = [content[0] + 4.0, ty, NPC_PORTRAIT_SIZE[0], NPC_PORTRAIT_SIZE[1]];
            let inner = npc_portrait_inner(card);
            r.push_clip(inner);
            if let Some(back) = lookups.ui_bestiary_slot_back {
                push_whole(r, lookups, &back, inner, [1.0; 4]);
            }
            // The biome backdrop (`UIBestiaryNPCEntryPortrait.cs:30-41`):
            // the preference-picked provider's image, `ScaleToFit` over the
            // inner box, tinted by the portrait color. W4-e23 remainder.
            if let Some((sheet_name, tint)) = portrait_backdrop(entry.net_id) {
                if let Some(sheet) = lookups.map_bg_sheet(sheet_name) {
                    push_whole(r, lookups, &sheet, inner, super::color::tint(tint));
                }
            }
            if state.known() {
                draw_portrait(r, lookups, entry, inner, true);
            } else if let Some(locked) = lookups.ui_bestiary_locked {
                let (lw, lh) = (locked.width as f32, locked.height as f32);
                push_whole(
                    r,
                    lookups,
                    &locked,
                    [inner[0] + (inner[2] - lw) * 0.5, inner[1] + (inner[3] - lh) * 0.5, lw, lh],
                    [0.15; 4],
                );
            }
            r.pop_clip();
            if let Some(front) = lookups.ui_bestiary_portrait {
                push_whole(r, lookups, &front, card, [1.0; 4]);
            }
            // The rarity stars (`CreateStarsContainer`, `:106-145`).
            if state.known() {
                draw_stars(r, lookups, [card[0] + 10.0, card[1] + 6.0], crate::bestiary_data::rarity_stars(entry));
            }
        }
        InfoRow::Flavor(flavor) => {
            // `FlavorTextBestiaryInfoElement.cs:20-53`: the wrapped 0.8 row
            // in the (43, 56, 101) panel, `Width (-11, 1)`.
            let panel_w = content[2] - 11.0;
            let lines = wrap_text(r, lookups, flavor, 0.8, panel_w - 8.0);
            let line_h = text::line_height(lookups) * 0.8;
            let h = lines.len() as f32 * line_h + 8.0;
            r.push_ui_sprite([content[0], ty], [panel_w, h], r.white_uv(), super::color::tint([43, 56, 101, 255]));
            for (i, line) in lines.iter().enumerate() {
                text::hud_text(r, lookups, content[0] + 4.0, ty + 4.0 + i as f32 * line_h, 0.8, [1.0; 4], line);
            }
        }
        InfoRow::Stats => {
            let stats =
                crate::npc::entry_stats(entry.kind, entry.variant.unwrap_or(SlimeVariant::Blue));
            let known = state >= UnlockState::CanShowStats;
            draw_stats_block(r, lookups, [tx, ty], content[2], &stats, known, screen, mouse);
        }
        InfoRow::KillCounter(count) => {
            draw_kill_counter(r, lookups, content, ty, count, screen, mouse);
        }
        InfoRow::Drop { name, rate, locked } => {
            // `UIBestiaryInfoItemLine`: below tier 3 the locked icon,
            // tier 3 "???", tier 4 stack + rate.
            if *locked {
                if let Some(locked_sheet) = lookups.ui_bestiary_locked {
                    let lw = locked_sheet.width as f32 * 0.55;
                    let lh = locked_sheet.height as f32 * 0.55;
                    push_whole(r, lookups, &locked_sheet, [tx + (content[2] - lw) * 0.5, ty + (32.0 - lh) * 0.5, lw, lh], [1.0; 4]);
                }
            } else {
                text::hud_text(r, lookups, tx, ty, 0.8, [1.0; 4], &format!("  {name} {rate}"));
            }
        }
        InfoRow::Separator => {
            // `UIHorizontalSeparator`, `(89, 116, 213) * 0.9` (`:153-157`).
            r.push_ui_sprite(
                [content[0], ty],
                [content[2], INFO_SEPARATOR_H],
                r.white_uv(),
                super::color::tint(super::color::mul([89, 116, 213, 255], 0.9)),
            );
        }
    }
}

/// `NPCPortraitInfoElement.CreateStarsContainer` (`:106-145`): the 5-star
/// row, `num4 = 14 + (-4) = 10` pitch, star `i` at `Left = 10*(i%5) -
/// num8*0.5 + 7`, `Top = 10*(i/5) - num9*0.5 + 7` (one row: `num8 = 46`,
/// `num9 = 6`), `HAlign/VAlign 0.5`. `Icon_Rank_Light` below the filled
/// count, `Icon_Rank_Dim` at/above it (`:131-135`). The container's
/// `(Left 10, Top 6)` offset is applied by the caller; `origin` here is the
/// container's top-left.
fn draw_stars(r: &mut SpriteRenderer, lookups: &Lookups, origin: [f32; 2], filled: i32) {
    let num8 = 46.0; // 10 * min(5,5) - (-4)
    let num9 = 6.0; // 10 * ceil(5/5) - (-4)
    for i in 0..5 {
        let sheet = if i < filled {
            lookups.ui_bestiary_star_light
        } else {
            lookups.ui_bestiary_star_dim
        };
        let Some(sheet) = sheet else { continue };
        let left = 10.0 * (i % 5) as f32 - num8 * 0.5 + 7.0;
        let top = 10.0 * (i / 5) as f32 - num9 * 0.5 + 7.0;
        // HAlign/VAlign 0.5 centres the 14x14 star on (left, top).
        let dst = [origin[0] + left - 7.0, origin[1] + top - 7.0, 14.0, 14.0];
        push_whole(r, lookups, &sheet, dst, [1.0; 4]);
    }
}

/// `UIText.IsWrapped`'s word wrap (`UIText.cs`'s `WordWrapping` pass): the
/// words greedily fill `max_w` at `scale`, one line per overflow; a word
/// longer than the line rides its own line (the vanilla breaks mid-word
/// only through `TrimTextIfNeeded`, not the wrap). At least one line.
fn wrap_text(
    r: &SpriteRenderer,
    lookups: &Lookups,
    text_in: &str,
    scale: f32,
    max_w: f32,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text_in.split_whitespace() {
        let candidate = if current.is_empty() {
            word.to_string()
        } else {
            format!("{current} {word}")
        };
        if !current.is_empty() && text::hud_text_width(r, lookups, &candidate, scale) > max_w {
            lines.push(std::mem::take(&mut current));
            current = word.to_string();
        } else {
            current = candidate;
        }
    }
    if !current.is_empty() || lines.is_empty() {
        lines.push(current);
    }
    lines
}

/// `NPCStatsReportInfoElement.ProvideUIElement`'s draw
/// (`NPCStatsReportInfoElement.cs:64-256`): the 2x2 icon grid with the
/// value texts right-aligned inside each icon (`HAlign 1`, `Left -10`,
/// scale 1), the coin separator + row when the value is positive
/// (denomination texts at scale 0.85, `Left -7`), and the four hover
/// tooltips (`ShowStats_Life/Attack/Defense/Knockback`, `:259-289`).
fn draw_stats_block(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    origin: [f32; 2],
    block_w: f32,
    stats: &crate::npc::EntryStats,
    known: bool,
    screen: Vec2,
    mouse: Vec2,
) {
    let masked = |v: String| if known { v } else { "???".to_string() };
    let icons = stats_grid_rects(origin);
    let sheets = [
        lookups.ui_bestiary_stat_hp,
        lookups.ui_bestiary_stat_attack,
        lookups.ui_bestiary_stat_defense,
        lookups.ui_bestiary_stat_knockback,
    ];
    let values = [
        masked(format!("{}", stats.hp)),
        masked(format!("{}", stats.damage)),
        masked(format!("{}", stats.defense)),
        knockback_label(stats.kb, known),
    ];
    // The hover labels (`:252-255` OnUpdate order: Attack, Defense, Life,
    // Knockback — the first hovered icon wins the MouseText).
    let hover_keys = [
        "BestiaryInfo.Life",
        "BestiaryInfo.Attack",
        "BestiaryInfo.Defense",
        "BestiaryInfo.Knockback",
    ];
    let mut hover: Option<&str> = None;
    for i in 0..4 {
        if let Some(sheet) = sheets[i] {
            push_whole(r, lookups, &sheet, icons[i], [1.0; 4]);
        }
        let w = text::hud_text_width(r, lookups, &values[i], 1.0);
        let p = stat_value_pos(icons[i], w, 1.0, -10.0); // `num5` (`:97`)
        text::hud_text(r, lookups, p[0], p[1], 1.0, [1.0; 4], &values[i]);
        if hover.is_none() && super::in_rect(mouse, icons[i]) {
            hover = Some(hover_keys[i]);
        }
    }
    if stats.value > 0.0 {
        // The coin separator (`:168-174`): `Width (0, 1)` — the block's
        // full width — at Top 70, the same `(89, 116, 213) * 0.9` the list
        // separators use.
        r.push_ui_sprite(
            [origin[0], origin[1] + 70.0],
            [block_w, 2.0],
            r.white_uv(),
            super::color::tint(super::color::mul([89, 116, 213, 255], 0.9)),
        );
        let coins = coin_strings(stats.value, known);
        let rects = coin_row_rects(origin);
        let coin_sheets = [
            lookups.ui_bestiary_stat_platinum,
            lookups.ui_bestiary_stat_gold,
            lookups.ui_bestiary_stat_silver,
            lookups.ui_bestiary_stat_copper,
        ];
        for i in 0..4 {
            if coins[i] == "-" {
                continue; // `:199-214` — the icon only appends when shown
            }
            if let Some(sheet) = coin_sheets[i] {
                push_whole(r, lookups, &sheet, rects[i], [1.0; 4]);
            }
            let w = text::hud_text_width(r, lookups, &coins[i], 0.85);
            let p = stat_value_pos(rects[i], w, 0.85, -7.0); // `num12` (`:215`)
            text::hud_text(r, lookups, p[0], p[1], 0.85, [1.0; 4], &coins[i]);
        }
    }
    if let Some(key) = hover {
        super::tooltip::draw_mouse_text(
            r,
            lookups,
            mouse,
            &crate::lang::text(key),
            screen,
            super::UiPulse::default(),
        );
    }
}

/// `NPCKillCounterInfoElement.ProvideUIElement`'s draw
/// (`NPCKillCounterInfoElement.cs:54-87`): the `Stat_Panel` panel
/// (12/7 splices, bg `(43, 56, 101)`, border transparent), the item-321
/// grave icon at scale 0.8 in a 32x32 `UIItemIcon` (`HAlign 0`,
/// `Left -1`), the count text right-aligned at scale 0.85 (`Left -3`,
/// `PaddingRight 5`), and the "Slain" hover (`ShowDescription`, `:90-96`).
fn draw_kill_counter(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    content: [f32; 4],
    row_top: f32,
    count: &str,
    screen: Vec2,
    mouse: Vec2,
) {
    let panel = kill_counter_panel_rect(content, row_top);
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.ui_bestiary_stat_panel,
        panel,
        12.0,
        12.0,
        7.0,
        7.0,
        super::color::tint([43, 56, 101, 255]),
    );
    // The grave icon (`:75-83`): `UIItemIcon` is a 32x32 box at HAlign 0,
    // Left -1 inside the panel; `DrawItemIcon` centres the item texture on
    // the box centre at `item.scale = 0.8` with the 32 px size limit.
    let icon_box = [panel[0] - 1.0, panel[1], 32.0, 32.0];
    let centre = (icon_box[0] + 16.0, icon_box[1] + 16.0);
    if let Some((uv, iw, ih)) = lookups.item_icon(321) {
        let longest = iw.max(ih);
        let limit = if longest > 32.0 { 32.0 / longest } else { 1.0 };
        let s = 0.8 * limit;
        let (w, h) = (iw * s, ih * s);
        r.push_ui_sprite([centre.0 - w * 0.5, centre.1 - h * 0.5], [w, h], uv, [1.0; 4]);
    }
    // The count (`:68-74`): HAlign 1 inside the panel's inner box
    // (PaddingRight 5), Left -3, VAlign 0.5, scale 0.85.
    let w = text::hud_text_width(r, lookups, count, 0.85);
    let inner_right = panel[0] + panel[2] - 5.0;
    let ty = panel[1] + (panel[3] - 16.0 * 0.85) * 0.5 - 2.0 * 0.85;
    text::hud_text(r, lookups, inner_right - w - 3.0, ty, 0.85, [1.0; 4], count);
    if super::in_rect(mouse, panel) {
        super::tooltip::draw_mouse_text(
            r,
            lookups,
            mouse,
            &crate::lang::text("BestiaryInfo.Slain"),
            screen,
            super::UiPulse::default(),
        );
    }
}

/// `Utils.Frame(horizontalFrames, verticalFrames, frameX, frameY)`
/// (`Utils.cs:1344-1349`): `w = tex.Width / horizontalFrames`,
/// `h = tex.Height / verticalFrames`, rect `(w * frameX, h * frameY, w, h)`
/// in **integer** division (the same helper `ui::chrome::frame` ports). The
/// linear `frame` unpacks as `frameX = frame % frames_x`,
/// `frameY = frame / frames_x`, so a horizontal 2-frame strip
/// (`Images/UI/Bestiary`, `Images/UI/Emotes`) slices a **column** half:
/// `w / 2`, the full height. The old single-frame vertical slice read the
/// whole strip for frame 0 (both button frames squashed into the 30x30 box)
/// and one full sheet height past the sheet's bottom for the hover frame,
/// which is the squeezed-button half of `BUG_REPORT.md` §14.
fn sheet_frame_uv(
    sheet: &tr_render::SheetRect,
    atlas: (u32, u32),
    frames_x: usize,
    frames_y: usize,
    frame: usize,
) -> [f32; 4] {
    let cols = frames_x.max(1);
    let rows = frames_y.max(1);
    let w = sheet.width / cols as u32;
    let h = sheet.height / rows as u32;
    let x0 = sheet.x + (frame % cols) as u32 * w;
    let y0 = sheet.y + (frame / cols) as u32 * h;
    [
        x0 as f32 / atlas.0 as f32,
        y0 as f32 / atlas.1 as f32,
        (x0 + w) as f32 / atlas.0 as f32,
        (y0 + h) as f32 / atlas.1 as f32,
    ]
}

/// One frame of a multi-frame sheet drawn into `dst` — the atlas-space
/// source rect is the `(frames_x, frames_y, frame)` cell of the sheet
/// (`sheet_frame_uv`).
fn draw_sheet_frame(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    sheet: &tr_render::SheetRect,
    frames_x: usize,
    frames_y: usize,
    frame: usize,
    dst: [f32; 4],
    tint: [f32; 4],
) {
    let uv = sheet_frame_uv(sheet, lookups.atlas_size, frames_x, frames_y, frame);
    r.push_ui_sprite([dst[0], dst[1]], [dst[2], dst[3]], uv, tint);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_page_offset_multiplies_the_capacity() {
        // `OffsetLibraryByPages` (`UIBestiaryEntryGrid.cs:103-107`) +
        // `FixBestiaryRange` (`:116-123`): 2 pages of 20 over 55 entries
        // clamps to 35 (55 - 20), never past the last full page.
        assert_eq!(offset_library_by_pages(0, 2, 20, 55), 35);
        assert_eq!(offset_library_by_pages(10, -1, 20, 55), 0);
        // A roster smaller than a page pins to 0.
        assert_eq!(offset_library_by_pages(0, 3, 20, 5), 0);
    }

    #[test]
    fn the_fractional_cell_placement_centres_the_lattice() {
        // `FillBestiarySpaceWithEntries` (`:50-66`): a 200x150 rect fits
        // 2x2 cells; the fractional offsets centre the 144x144 lattice —
        // cell (0,0) at (28, 3), cell (1,1) at (100, 75).
        let rect = [0.0, 0.0, 200.0, 150.0];
        assert_eq!(grid_cell_rect(&rect, 2, 2, 0, 0), [28.0, 3.0, 72.0, 72.0]);
        assert_eq!(grid_cell_rect(&rect, 2, 2, 1, 1), [100.0, 75.0, 72.0, 72.0]);
        // An exact-multiple rect matches the raw CELL lattice the draw uses.
        let rect = [10.0, 20.0, 144.0, 144.0];
        assert_eq!(grid_cell_rect(&rect, 2, 2, 1, 0), [82.0, 20.0, 72.0, 72.0]);
    }

    #[test]
    fn the_border_layers_swap_on_hover_and_drop_for_portraits() {
        // `MouseOver`/`MouseOut` (`UIBestiaryEntryButton.cs:150-169`).
        assert_eq!(entry_button_border_layers(false, false), (true, false));
        assert_eq!(entry_button_border_layers(true, false), (false, true));
        // The pretty-portrait ctor removes the overlay (`:93-96`).
        assert_eq!(entry_button_border_layers(false, true), (false, false));
        assert_eq!(entry_button_border_layers(true, true), (false, false));
    }

    #[test]
    fn the_background_pick_prefers_matched_providers() {
        // `TryGettingBackgroundImageProvider` (`:104-129`): the matched
        // provider with an image wins over an earlier unmatched one.
        let providers = [(false, true), (true, false), (true, true)];
        assert_eq!(pick_background_provider(&providers), Some(2));
        // No matched image: the first image in entry order (`:120-127`).
        assert_eq!(
            pick_background_provider(&[(false, false), (false, true)]),
            Some(1)
        );
        assert_eq!(
            pick_background_provider(&[(true, false), (false, false)]),
            None
        );
        assert_eq!(pick_background_provider(&[]), None);
    }

    #[test]
    fn the_display_index_is_the_first_info_element() {
        // `TryGettingDisplayIndex` (`:131-140`): FirstOrDefault over Info.
        assert_eq!(first_display_index(&[None, Some(7), Some(9)]), Some(7));
        assert_eq!(first_display_index(&[None, None]), None);
        assert_eq!(first_display_index(&[]), None);
    }

    #[test]
    fn the_icon_hover_and_walk_gates_follow_the_source() {
        // `UIBestiaryEntryIcon.cs:38`/`:52`: IsMouseHovering || ForceHover.
        assert!(icon_is_hovered(false, true));
        assert!(icon_is_hovered(true, false));
        assert!(!icon_is_hovered(false, false));
        // `UnlockableNPCEntryIcon.cs:100-109`: no Frame override + (portrait
        // or hovered) walks; a Frame override pins instead.
        assert!(icon_walks(None, false, true));
        assert!(icon_walks(None, true, false));
        assert!(!icon_walks(None, false, false));
        assert!(!icon_walks(Some(2), true, true));
    }

    #[test]
    fn the_info_page_scrollbar_box_and_attach_machine() {
        // `UIBestiaryEntryInfoPage.cs:47-52`: Height -20, VAlign 0.5,
        // HAlign 1, Left -6 inside the 230 px page.
        let page = [100.0, 50.0, 230.0, 400.0];
        assert_eq!(
            info_page_scrollbar_rect(page, 20.0),
            [304.0, 60.0, 20.0, 380.0]
        );
        // `UpdateScrollbar` (`:59-65`): ViewPosition -= wheel.
        assert_eq!(info_page_scroll(100.0, 120.0), -20.0);
        // `CheckScrollBar` (`:90-109`): attach shrinks the list by 20.
        assert_eq!(info_page_check_scrollbar(false, true), (true, -20.0));
        assert_eq!(info_page_check_scrollbar(true, false), (false, 0.0));
        assert_eq!(info_page_check_scrollbar(true, true), (true, -20.0));
        assert_eq!(info_page_check_scrollbar(false, false), (false, 0.0));
        // The ctor colours (`:33-34`) carry no opacity multiply.
        assert_eq!(
            info_page_colors(),
            ([89, 116, 213, 255], [73, 94, 171, 255])
        );
    }

    #[test]
    fn the_info_row_order_groups_by_category_then_priority() {
        // `AddInfoToList` (`:128-163`): categories ascending, priorities
        // descending within a group, separators between groups only.
        use BestiaryInfoElementKind as K;
        let elements = [
            (K::ItemDrop, 0.0, true),       // 0: ItemsFromDrops
            (K::NpcPortrait, 0.0, true),    // 1: Portrait
            (K::NpcStatsReport, 5.0, true), // 2: Stats, high priority
            (K::NpcKillCounter, 0.0, true), // 3: Stats, low priority
            (K::FlavorText, 0.0, false),    // 4: produced nothing - skipped
            (K::NamePlate, 0.0, true),      // 5: Nameplate
        ];
        let (order, separators) = info_page_row_order(&elements);
        // Nameplate(0) < Portrait(1) < Stats(3) < ItemsFromDrops(5).
        assert_eq!(order, vec![5, 1, 2, 3, 0]);
        // Separators after the first three groups, none after the last.
        assert_eq!(separators, vec![1, 2, 4]);
        // Ties keep entry order (stable OrderByDescending).
        let elements = [(K::ItemDrop, 1.0, true), (K::ItemDrop, 1.0, true)];
        assert_eq!(info_page_row_order(&elements).0, vec![0, 1]);
        // The category dispatch (`:174-201`).
        assert_eq!(
            bestiary_info_category(K::NpcKillCounter),
            BestiaryInfoCategory::Stats
        );
        assert_eq!(bestiary_info_category(K::Other), BestiaryInfoCategory::Misc);
    }

    #[test]
    fn the_filter_grid_pins_the_hard_overwritten_settings() {
        // `GetDisplaySettings` (`UIBestiaryFilteringOptionsGrid.cs:159-173`):
        // perRow 12 (the sqrt walk dead), offsets 6 (the centring dead).
        assert_eq!(filter_grid_display_settings(25), (12, 3, 6.0, 6.0));
        assert_eq!(filter_grid_display_settings(0), (12, 0, 6.0, 6.0));
        // `BuildContainer` (`:46-49`): 12*34+10 = 418 wide, rows*34+10.
        assert_eq!(filter_grid_container_size(25), [418.0, 112.0]);
        // Button 13 sits on row 1 (`:90-102`).
        assert_eq!(filter_button_top_left(0), [6.0, 6.0]);
        assert_eq!(filter_button_top_left(13), [40.0, 40.0]);
        // `UpdateButtonSelections` (`:175-190`).
        assert_eq!(
            filter_button_selection(true, 4),
            ([152, 175, 235, 255], 1.0, 4)
        );
        assert_eq!(
            filter_button_selection(false, 4),
            (uistate::INVENTORY_DEFAULT_COLOR, 0.7, -1)
        );
        // `GetIsFilterAvailableForEntries` (`:192-207`).
        assert!(filter_is_available(Some(true), &[]));
        assert!(!filter_is_available(Some(false), &[(true, true)]));
        assert!(filter_is_available(None, &[(false, true), (true, true)]));
        assert!(!filter_is_available(None, &[(true, false)]));
        // The unavailable button (`:118-146`): option -2, fade 0.5, the
        // shadow icon at White * 0.2 = (51, 51, 51, 51).
        assert_eq!(unavailable_filter_button(), (-2, 0.5, [51, 51, 51, 51]));
    }

    #[test]
    fn the_info_item_line_geometry_and_hover_pair() {
        // The ctor (`UIBestiaryInfoItemLine.cs:26-31`): Width -14, Left 5,
        // Height 32.
        let content = [100.0, 50.0, 200.0, 32.0];
        assert_eq!(info_item_line_rect(content), [105.0, 50.0, 186.0, 32.0]);
        // The locked icon at 0.55 scale, centred (`:36-55`): a 40x40 texture
        // draws 22x22 at the line's centre.
        let line = [105.0, 50.0, 186.0, 32.0];
        assert_eq!(
            info_item_locked_icon_rect(line, [40.0, 40.0]),
            [187.0, 55.0, 22.0, 22.0]
        );
        // The border pair (`:34`, `:158-167`).
        assert_eq!(info_item_line_border(false), [89, 116, 213, 255]);
        assert_eq!(
            info_item_line_border(true),
            uistate::FANCY_UI_FAT_BUTTON_MOUSE_OVER
        );
        // The tooltip gate (`:112-119`): locked lines never show it.
        assert!(info_item_shows_tooltip(true, false));
        assert!(!info_item_shows_tooltip(true, true));
        assert!(!info_item_shows_tooltip(false, false));
    }

    #[test]
    fn the_bestiary_notes_join_the_non_empty_conditions() {
        // `SetBestiaryNotesOnItemCache` (`:137-156`): null and whitespace
        // descriptions drop, the rest join with "\n".
        assert_eq!(
            bestiary_notes(&[Some("Expert only"), None, Some("  "), Some("Master only")]),
            "Expert only\nMaster only"
        );
        assert_eq!(bestiary_notes(&[]), "");
        assert_eq!(bestiary_notes(&[None, Some("")]), "");
    }

    #[test]
    fn the_info_line_measures_and_centres_its_text() {
        // `SetText` (`UIBestiaryInfoLine.cs:73-81`): (measured, 16) * scale,
        // MinWidth/Height add the padding pair.
        let (size, min_w, min_h) = info_line_text_size(100.0, 0.8, [2.0, 4.0, 2.0, 4.0]);
        assert_eq!(size, [80.0, 12.8]);
        assert_eq!(min_w, 84.0);
        assert_eq!(min_h, 20.8);
        // `DrawSelf` (`:83-90`): centred X, Y rides 2*scale up.
        let pos = info_line_text_pos([100.0, 50.0, 200.0, 20.0], [80.0, 12.8], 0.8);
        assert_eq!(pos, [160.0, 48.4]);
    }

    #[test]
    fn the_npc_portrait_card_pins_the_ctor_geometry() {
        // `UIBestiaryNPCEntryPortrait.cs:17-28`: 193x112, the 2 px inset box.
        assert_eq!(NPC_PORTRAIT_SIZE, [193.0, 112.0]);
        let card = [0.0, 0.0, 193.0, 112.0];
        assert_eq!(npc_portrait_inner(card), [2.0, 2.0, 189.0, 108.0]);
        // The overlay fallback (`:53`): Lerp(White, portraitColor, 0.5).
        let portrait = [63, 65, 151, 255];
        assert_eq!(
            npc_portrait_overlay_color(Some([1, 2, 3, 4]), portrait),
            [1, 2, 3, 4]
        );
        assert_eq!(
            npc_portrait_overlay_color(None, portrait),
            super::super::char_creation::lerp_rgb([255, 255, 255, 255], portrait, 0.5)
        );
    }

    #[test]
    fn the_sort_grid_lays_out_the_visible_steps() {
        // `BuildGrid` (`UIBestiarySortingOptionsGrid.cs:37-74`): 126 wide,
        // count*28+8 tall, buttons 114x26 at Top 5+28n, HAlign 0.5.
        assert_eq!(sort_grid_container_size(4), [126.0, 120.0]);
        let container = [100.0, 50.0, 126.0, 120.0];
        assert_eq!(sort_button_rect(container, 0), [106.0, 55.0, 114.0, 26.0]);
        assert_eq!(sort_button_rect(container, 2), [106.0, 111.0, 114.0, 26.0]);
        // `ClickOption` (`:89-115`): re-click falls back to the default.
        assert_eq!(sort_click_option(3, 3, 0), 0);
        assert_eq!(sort_click_option(2, 3, 0), 2);
        // The colour pair is the filtering grid's shared walk (`:100-107`).
        assert_eq!(sort_button_colors(true), ([152, 175, 235, 255], 1.0));
        assert_eq!(
            sort_button_colors(false),
            (uistate::INVENTORY_DEFAULT_COLOR, 0.7)
        );
        // `GetEntriesToShow` (`:117-122`): one column.
        assert_eq!(sort_grid_entries_to_show(4), (1, 4, 4));
    }

    #[test]
    fn dw_3_1_panel_rect_is_the_c_sharp_container() {
        // `UIBestiaryTest.cs:71-79` at the 1600x900 reference: width 0.875*1600
        // capped to 900, centred; Top 180; Height = screenH - 220 = 680, so the
        // panel ends at 860 (screenH - 40).
        let p = panel_rect(Vec2::new(1600.0, 900.0));
        assert_eq!(p, [350.0, 180.0, 900.0, 680.0]);
        // 1280x720: the same cap, height 720 - 220 = 500.
        let p = panel_rect(Vec2::new(1280.0, 720.0));
        assert_eq!(p, [190.0, 180.0, 900.0, 500.0]);
        // A narrow screen clamps to `MinWidth` 700 (`:73`, 600 + 100) — not the
        // old hand-written 560 floor.
        let p = panel_rect(Vec2::new(640.0, 480.0));
        assert_eq!(p[2], 700.0);
        // The Back panel still sits inside the container.
        let back = back_rect(&p);
        assert!(back[1] + back[3] <= p[1] + p[3]);
    }

    #[test]
    fn the_grid_indexes_match_the_layout() {
        // A 1280x720 screen: the panel is 900 wide (0.875*1280 = 1120 capped);
        // the bestiary space fits 8x4 = 32 cells and the roster PAGES
        // (`OffsetLibraryByPages`), it no longer grows its rows.
        let screen = Vec2::new(1280.0, 720.0);
        let panel = panel_rect(screen);
        assert_eq!(panel[2], 900.0);
        let rect = bestiary_space(&panel);
        let (cols, rows) = grid_metrics(&rect);
        assert_eq!((cols, rows), (8, 4));
        // The first cell's centre hits slot 0 — the probe rides the raw
        // CELL lattice (no insets); the slot maps through the page start.
        let first_x = rect[0] + CELL * 0.5;
        let first_y = rect[1] + CELL * 0.5;
        assert_eq!(grid_slot_at(screen, Vec2::new(first_x, first_y)), Some(0));
        // Past the space's right edge: no slot (the info page's column).
        assert_eq!(
            grid_slot_at(screen, Vec2::new(rect[0] + rect[2] + 6.0, first_y)),
            None
        );
        // The Back panel sits inside the container above the bottom edge.
        let back = back_rect(&panel);
        assert!(back[1] + back[3] <= panel[1] + panel[3]);
        assert!(back_hit(screen, Vec2::new(back[0] + 10.0, back[1] + 10.0)));
        // The inventory button's hover rect matches the vanilla constants.
        assert!(button_hit(Vec2::new(498.0, 278.0)));
        assert!(!button_hit(Vec2::new(490.0, 278.0)));
    }

    /// A `Lookups` with the full 174-slot `bestiary_npc_sheets` table
    /// (`Lookups::default()` leaves it empty) — the exact runtime length, so
    /// an index past the table's end panics the way the Solar Selenian arm's
    /// `[174]` did (`BUG_REPORT.md` §15). The sheet sizes are a stand-in for
    /// the content: `NPC_419`'s real 8-row strip.
    fn full_bestiary_lookups() -> Lookups {
        let mut lk = Lookups::default();
        lk.atlas_size = (8192, 8192);
        lk.bestiary_npc_sheets = crate::assets::BESTIARY_NPC_SHEET_IDS
            .iter()
            .map(|_| {
                Some(tr_render::SheetRect {
                    x: 0,
                    y: 0,
                    width: 22,
                    height: 8 * 56,
                    plain_frame: (0, 0),
                })
            })
            .collect();
        lk
    }

    #[test]
    fn every_bestiary_sheet_slot_opens_its_page_without_panicking() {
        // §15: the 174-entry loop over `BESTIARY_NPC_SHEET_IDS` — one open
        // page per row of the runtime table. The Solar Selenian arm used to
        // read `bestiary_npc_sheets[174]`, one past the 174-entry table
        // (`assets.rs`'s `[u16; 174]`), and panicked the page; every arm now
        // derives its sheet from the entry's own vanilla `net_id` through
        // `Lookups::bestiary_sheet_index` (the helper at `assets.rs:1512`).
        let lk = full_bestiary_lookups();
        assert_eq!(
            lk.bestiary_npc_sheets.len(),
            crate::assets::BESTIARY_NPC_SHEET_IDS.len()
        );
        for &net_id in crate::assets::BESTIARY_NPC_SHEET_IDS.iter() {
            let slot = lk
                .bestiary_sheet_index(net_id)
                .unwrap_or_else(|| panic!("net {net_id} resolves no sheet slot"));
            assert!(
                slot < crate::assets::BESTIARY_NPC_SHEET_IDS.len(),
                "slot {slot} for net {net_id} must be inside the table"
            );
            // `NPCID.SolarSolenian` = 419 (`NPCID.cs:11515`) — the arm whose
            // page panicked. Its derived slot must be a real row, not the
            // one-past-the-end 174 the literal read.
            if net_id == 419 {
                assert_eq!(slot, 135);
                assert!(slot + 1 < crate::assets::BESTIARY_NPC_SHEET_IDS.len());
            }
            // The catalog may not carry an entry for every sheet (the port
            // registers only part of the vanilla NPC set); where it does, the
            // page's portrait arm (the one that indexed the vec directly)
            // must resolve and produce sane frame metrics.
            if let Some(entry) = crate::bestiary::ENTRIES
                .iter()
                .find(|e| e.net_id == net_id as i32)
            {
                assert!(entry_sheet(&lk, entry).is_some(), "net {net_id} sheet");
                let (_sheet_i, frame_h, w, h, tint) = portrait_data(entry, &lk);
                assert!(
                    frame_h > 0.0 && w > 0.0 && h > 0.0,
                    "net {net_id} portrait {frame_h} x {w} x {h}"
                );
                assert!(tint.iter().all(|c| c.is_finite()), "net {net_id} tint");
            }
        }
    }

    #[test]
    fn the_solar_selenian_page_resolves_its_own_sheet() {
        // The Solar Selenian (`NPCID.SolarSolenian` = 419, `NPCID.cs:11515`;
        // the port's `credit_id: "SolarSelenian"`, `bestiary.rs:1238`) opens
        // through the derived helper: the 8-row `NPC_419` strip gives a
        // 22x56 portrait over the sheet's own frame height.
        let lk = full_bestiary_lookups();
        let entry = crate::bestiary::ENTRIES
            .iter()
            .find(|e| e.net_id == 419)
            .expect("the Selenian bestiary entry");
        let sheet = entry_sheet(&lk, entry).expect("the Selenian's own sheet");
        assert_eq!(sheet.height, 8 * 56);
        let (_sheet_i, frame_h, w, h, _tint) = portrait_data(entry, &lk);
        assert_eq!((frame_h, w, h), (56.0, 22.0, 56.0));
    }

    #[test]
    fn every_catalog_entry_survives_the_portrait_arm() {
        // The other direction: every entry in the catalog (including the
        // slime variants' negative net ids, which have no sheet slot and so
        // skip the portrait) resolves without an out-of-range index.
        let lk = full_bestiary_lookups();
        for entry in crate::bestiary::ENTRIES.iter() {
            let (_sheet_i, frame_h, w, h, tint) = portrait_data(entry, &lk);
            assert!(
                frame_h > 0.0 && w > 0.0 && h > 0.0,
                "{} portrait",
                entry.name
            );
            assert!(tint.iter().all(|c| c.is_finite()), "{} tint", entry.name);
            if entry.net_id >= 0 {
                // A positive net id resolves a sheet exactly when it is one of
                // the table's 174 rows (the table drives `bestiary_sheet_index`).
                let tabled = crate::assets::BESTIARY_NPC_SHEET_IDS.contains(&(entry.net_id as u16));
                assert_eq!(entry_sheet(&lk, entry).is_some(), tabled, "{}", entry.name);
            }
        }
    }

    #[test]
    fn portrait_source_rect_is_one_animation_frame() {
        // `UnlockableNPCEntryIcon.Draw` (`UnlockableNPCEntryIcon.cs:194-207`)
        // draws the NPC's single `npc.frame`, not the whole animation sheet.
        // The 8-row NPC_419 strip at frame height 56 must slice to 8 rows,
        // and frame 0's source rect covers exactly the sheet's first 56 px.
        let sheet = tr_render::SheetRect {
            x: 100,
            y: 200,
            width: 22,
            height: 8 * 56,
            plain_frame: (0, 0),
        };
        assert_eq!(portrait_frames_y(sheet.height, 56.0), 8);
        // A single-frame sheet stays whole; a bad frame height falls back to 1.
        assert_eq!(portrait_frames_y(48, 48.0), 1);
        assert_eq!(portrait_frames_y(448, 0.0), 1);
        let uv = sheet_frame_uv(&sheet, (8192, 8192), 1, 8, 0);
        assert_eq!(
            uv,
            [
                100.0 / 8192.0,
                200.0 / 8192.0,
                122.0 / 8192.0,
                256.0 / 8192.0
            ]
        );
    }

    #[test]
    fn the_draw_offset_table_carries_the_vanilla_rows() {
        use super::super::bestiary_draw_offsets::get;
        // KingSlime (50): `Position (0, 90)`, `PortraitScale 1.1`,
        // `PortraitPositionYOverride 70` (`NPCID.cs:1585-1592`).
        let ks = get(50).expect("KingSlime row");
        assert_eq!(ks.position, [0.0, 90.0]);
        assert_eq!(ks.scale, 1.0);
        assert_eq!(ks.portrait_scale, Some(1.1));
        assert_eq!(ks.portrait_position_y_override, Some(70.0));
        // MoonLordCore (398): `Scale 0.4`, `PortraitScale 0.7`, the
        // dedicated portrait texture (`NPCID.cs:3880-3890`).
        let ml = get(398).expect("MoonLordCore row");
        assert_eq!(ml.scale, 0.4);
        assert_eq!(ml.portrait_scale, Some(0.7));
        assert_eq!(ml.position, [0.0, 5.0]);
        assert_eq!(ml.portrait_position_y_override, Some(0.0));
        assert_eq!(
            ml.custom_texture_path,
            Some("Images/UI/Bestiary/NPCs/NPC_398")
        );
        // The four towers at `Scale 0.4` (`NPCID.cs:4033-4043`,
        // `:4494-4504`, `:4596-4606`, `:4679-4689`).
        for id in [422, 493, 507, 517] {
            let t = get(id).unwrap_or_else(|| panic!("tower {id} row"));
            assert_eq!(t.scale, 0.4, "tower {id} scale");
            assert_eq!(t.position, [1.0, 44.0], "tower {id} position");
        }
        // An id without a row resolves `None` (the `TryGetValue` miss).
        assert!(get(1).is_none());
    }

    #[test]
    fn the_eye_of_cthulhu_icon_uses_frame_4() {
        // EoC (4): `Position (25, -30)`, `Rotation 0.7`, `Frame 4`
        // (`NPCID.cs:1281-1289`) - `frame.Y = frame.Height * 4`
        // (`UnlockableNPCEntryIcon.cs:105-109`). EoC sets `noGravity = true`
        // (`NPC.cs:8714`), so the icon CENTRES (`UpdatePosition`, `:112-118`).
        let iconbox = [100.0, 100.0, 68.0, 68.0];
        let ([x, y], scale, frame) = portrait_layout(4, true, 100.0, 110.0, iconbox, false);
        assert_eq!(frame, 4);
        assert_eq!(scale, 1.0);
        // Centred anchor + offset, floored (`UpdatePosition`, `:112-122`):
        // x = 100 + (68-100)/2 + 25 = 109; y = 100 + (68-110)/2 - 30 = 49.
        assert_eq!([x, y], [109.0, 49.0]);
        // The generated NO_GRAVITY table carries EoC (and the towers/MoonLord).
        assert!(super::super::bestiary_draw_offsets::no_gravity(4));
        assert!(super::super::bestiary_draw_offsets::no_gravity(398));
        assert!(super::super::bestiary_draw_offsets::no_gravity(422));
        // Demon Eye (2) does NOT set noGravity in SetDefaults -> bottom-anchor.
        assert!(!super::super::bestiary_draw_offsets::no_gravity(2));
        // Compound-arm floater (fireflies 583-585, `type == 583 || 584 || 585`).
        assert!(super::super::bestiary_draw_offsets::no_gravity(583));
        // Range-arm floater (`type >= 595 && type <= 601`).
        assert!(super::super::bestiary_draw_offsets::no_gravity(595));
        // AI-only non-floater: Hornet (48, aiStyle 14) sets noGravity only in
        // its runtime AI block, NOT SetDefaults -> bottom-anchor in the cache.
        assert!(!super::super::bestiary_draw_offsets::no_gravity(48));
    }

    #[test]
    fn portrait_scale_and_overrides_apply_per_mode() {
        // KingSlime's `PortraitScale 1.1` + `PortraitPositionYOverride 70`
        // apply only with `IsPortrait` (`UnlockableNPCEntryIcon.cs:56-59`,
        // `:70-73`); the grid icon keeps `Scale 1` and `Position (0, 90)`.
        let iconbox = [0.0, 0.0, 68.0, 68.0];
        let ([_, gy], gs, _) = portrait_layout(50, false, 174.0, 120.0, iconbox, false);
        assert_eq!(gs, 1.0);
        assert_eq!(gy, (68.0f32 - 120.0 - 8.0 + 90.0).floor()); // 30
        let ([_, py], ps, _) = portrait_layout(50, false, 174.0, 120.0, iconbox, true);
        assert_eq!(ps, 1.1);
        assert_eq!(py, (68.0f32 - 120.0 - 8.0 + 70.0).floor()); // 10
                                                             // MoonLordCore: grid 0.4, portrait 0.7 (`NPCID.cs:3880-3890`).
        let (_, gs, _) = portrait_layout(398, false, 46.0, 66.0, iconbox, false);
        let (_, ps, _) = portrait_layout(398, false, 46.0, 66.0, iconbox, true);
        assert_eq!((gs, ps), (0.4, 0.7));
        // An id with no row draws at scale 1, frame 0, no offset.
        let ([x, y], s, f) = portrait_layout(1, false, 32.0, 26.0, iconbox, false);
        assert_eq!((s, f), (1.0, 0));
        assert_eq!([x, y], [18.0, 34.0]); // 34-16, 68-26-8
    }

    #[test]
    fn the_clip_rect_is_the_inner_68_box_inside_the_cell() {
        // `UIBestiaryEntryButton.cs:30-38`: the `OverflowHidden` container
        // is `Width/Height (-4, 1)` centred — the 68 px box inset 2 px per
        // side, fully inside the 72 px cell.
        let cell = [144.0, 216.0, CELL, CELL];
        let inner = inner_box(cell);
        assert_eq!(inner, [146.0, 218.0, 68.0, 68.0]);
        assert!(inner[0] >= cell[0] && inner[1] >= cell[1]);
        assert!(inner[0] + inner[2] <= cell[0] + cell[2]);
        assert!(inner[1] + inner[3] <= cell[1] + cell[3]);
        // The scissor stack intersects nested rects (`UIElement.cs:265-268`
        // via `ClipStack::push`, the stack behind `push_clip`) — the inner
        // box survives a full-screen clip and pops back to it.
        let mut stack = tr_render::ClipStack::new();
        stack.push([0.0, 0.0, 1600.0, 900.0]);
        stack.push(inner);
        assert_eq!(stack.current(), inner);
        stack.pop();
        assert_eq!(stack.current(), [0.0, 0.0, 1600.0, 900.0]);
        stack.pop();
    }

    #[test]
    fn the_two_inventory_buttons_never_overlap() {
        // §14: `Main.DrawBestiaryIcon` (498,278) and
        // `Main.DrawEmoteBubblesButton` (534,278) (`Main.cs:42423-42485`,
        // `:42369-42408`) — two 30x30 boxes with a 6 px gutter, drawn side by
        // side by `Main.DrawInventory` (`Main.cs:41012-41013`).
        assert_eq!(BUTTON_RECT, [498.0, 278.0, 30.0, 30.0]);
        assert_eq!(EMOTE_BUTTON_RECT, [534.0, 278.0, 30.0, 30.0]);
        assert_eq!(BUTTON_RECT[0] + BUTTON_RECT[2], 528.0, "bestiary 498..528");
        assert_eq!(
            EMOTE_BUTTON_RECT[0] + EMOTE_BUTTON_RECT[2],
            564.0,
            "emote 534..564"
        );
        assert!(BUTTON_RECT[0] + BUTTON_RECT[2] < EMOTE_BUTTON_RECT[0]);
        assert_eq!(
            EMOTE_BUTTON_RECT[0] - (BUTTON_RECT[0] + BUTTON_RECT[2]),
            6.0
        );
        // Same row: the two hover frames stay mutually exclusive at every
        // cursor position, edges included.
        for x in [
            497.0, 498.0, 512.0, 527.0, 528.0, 533.0, 534.0, 549.0, 564.0, 565.0,
        ] {
            let m = Vec2::new(x, 290.0);
            assert!(
                !(button_hit(m) && emote_button_hit(m)),
                "x {x} hits both buttons"
            );
        }
    }

    #[test]
    fn the_two_frame_buttons_slice_a_column_half_each() {
        // `value.Frame(2, 1, flag ? 1 : 0)` (`Main.cs:42464-42467`,
        // `:42408-42409`) is `Utils.Frame(2, 1, frameX, 0)` (`Utils.cs:1344`):
        // column `frameX` of a 2-column strip, so each frame is
        // `width / 2` wide and the full height. The old single-frame
        // vertical slice squashed both frames into the 30x30 button (§14).
        let sheet = tr_render::SheetRect {
            x: 40,
            y: 80,
            width: 60,
            height: 30,
            plain_frame: (0, 0),
        };
        let atlas = (1024u32, 1024u32);
        let normal = sheet_frame_uv(&sheet, atlas, 2, 1, 0);
        let hover = sheet_frame_uv(&sheet, atlas, 2, 1, 1);
        assert_eq!(
            normal,
            [40.0 / 1024.0, 80.0 / 1024.0, 70.0 / 1024.0, 110.0 / 1024.0]
        );
        assert_eq!(
            hover,
            [70.0 / 1024.0, 80.0 / 1024.0, 100.0 / 1024.0, 110.0 / 1024.0]
        );
        // Vertical strips keep splitting rows: `Frame(1, 3, 0, 1)` on a
        // 60x30 sheet is row 1 of three 60x10 rows.
        let row1 = sheet_frame_uv(&sheet, atlas, 1, 3, 1);
        assert_eq!(
            row1,
            [40.0 / 1024.0, 90.0 / 1024.0, 100.0 / 1024.0, 100.0 / 1024.0]
        );
        // `Frame(1, 1, 0)` is the whole sheet — the portrait path's mode.
        let whole = sheet_frame_uv(&sheet, atlas, 1, 1, 0);
        assert_eq!(
            whole,
            [40.0 / 1024.0, 80.0 / 1024.0, 100.0 / 1024.0, 110.0 / 1024.0]
        );
        // The button body: the frame shrunk 2 px per side and centred
        // (`rectangle2.Width -= 2`), so the emote button's 60x30 sheet
        // paints 28x28 at (535, 279).
        let dst = two_frame_dst(&sheet, EMOTE_BUTTON_RECT);
        assert_eq!(dst, [535.0, 279.0, 28.0, 28.0]);
        let dst = two_frame_dst(&sheet, BUTTON_RECT);
        assert_eq!(dst, [499.0, 279.0, 28.0, 28.0]);
    }

    // ---- S1-E-26 wave C: the grid range/clamp, the lock-state text gates
    // and the completion bar. Written in code-only mode — NOT RUN here.

    #[test]
    fn grid_range_text_matches_get_range_text() {
        // `UIBestiaryEntryGrid.GetRangeText` (`UIBestiaryEntryGrid.cs:87-93`):
        // $"{Min(at + 1, last)}-{last} ({total})" with
        // last = Min(total, at + capacity).
        assert_eq!(grid_range_text(0, 36, 174), "1-36 (174)");
        assert_eq!(grid_range_text(36, 36, 174), "37-72 (174)");
        // The last partial page: at 144, capacity 36, total 174 -> 145-174.
        assert_eq!(grid_range_text(144, 36, 174), "145-174 (174)");
        // A roster smaller than one page.
        assert_eq!(grid_range_text(0, 36, 12), "1-12 (12)");
        // The empty roster: Min(1, 0) = 0.
        assert_eq!(grid_range_text(0, 36, 0), "0-0 (0)");
    }

    #[test]
    fn fix_grid_range_clamps_like_utils_clamp() {
        // `FixBestiaryRange` (`UIBestiaryEntryGrid.cs:116-123`):
        // Clamp(at + offset, 0, Max(0, total - capacity)).
        // 174 entries, 36/page -> the last page starts at 138.
        assert_eq!(fix_grid_range(0, 36, 36, 174), 36);
        assert_eq!(fix_grid_range(138, 36, 36, 174), 138, "never past the last page");
        assert_eq!(fix_grid_range(0, -36, 36, 174), 0, "never below 0");
        assert_eq!(fix_grid_range(138, 3600, 36, 174), 138);
        // total < capacity pins the max to 0.
        assert_eq!(fix_grid_range(0, 36, 36, 12), 0);
        assert_eq!(fix_grid_range(0, 0, 36, 0), 0);
    }

    #[test]
    fn the_completion_bar_rides_the_bottom_strip() {
        // `FillProgressBottomBar` (`UIBestiaryTest.cs:158-179`): 15 px tall,
        // VAlign 1 in the 20 px bottom strip; the strip is VAlign 1 in the
        // panel's inner rect — the (-90, 1) panel height (`:83`) and the
        // 8 px bottom padding (`PaddingBottom -= 4`, `:87`) put the bar's
        // bottom at panelBottom - 90 - 8.
        let panel = panel_rect(Vec2::new(1600.0, 900.0));
        let bar = progress_bar_rect(&panel);
        assert_eq!(bar[3], 15.0);
        assert_eq!(bar[0], panel[0] + 12.0);
        assert_eq!(bar[2], panel[2] - 24.0);
        assert_eq!(bar[1] + bar[3], panel[1] + panel[3] - 90.0 - 8.0);
    }

    #[test]
    fn completion_counts_every_known_entry_not_only_tier_4() {
        // `GetUnlockProgress` (`UIBestiaryTest.cs:497-513`) counts
        // `UnlockState > NotKnownAtAll_0` — one kill of the first entry
        // already moves the bar (the old tier-4-only count did not).
        let mut tracker = WorldBestiary::new();
        let first = &crate::bestiary::ENTRIES[0];
        assert_eq!(crate::bestiary::completion_percent(&tracker), 0.0);
        tracker.kills.register_kill(first.credit_id);
        let pct = crate::bestiary::completion_percent(&tracker);
        assert!(pct > 0.0, "one kill must move the completion bar");
        assert_eq!(pct, 1.0 / crate::bestiary::ENTRIES.len() as f32);
    }

    #[test]
    fn hover_text_is_the_name_or_question_marks() {
        // `UnlockableNPCEntryIcon.GetHoverText`
        // (`UnlockableNPCEntryIcon.cs:209-221`): "???" below tier 1, the
        // NPC name from tier 1 up.
        let entry = &crate::bestiary::ENTRIES[0];
        assert_eq!(entry_hover_text(entry, UnlockState::NotKnownAtAll), "???");
        let name = entry_hover_text(entry, UnlockState::CanShowPortraitOnly);
        assert_ne!(name, "???");
        assert!(!name.is_empty());
    }

    #[test]
    fn kill_counter_needs_a_first_kill() {
        // `NPCKillCounterInfoElement.ProvideUIElement` (`:26-30`): null
        // below 1, the raw count string above (no "/needed" fraction).
        assert_eq!(kill_counter_text(0), None);
        assert_eq!(kill_counter_text(-3), None);
        assert_eq!(kill_counter_text(1), Some("1".to_string()));
        assert_eq!(kill_counter_text(999_999_999), Some("999999999".to_string()));
    }

    #[test]
    fn drop_rate_text_gates_by_tier() {
        // `UIBestiaryInfoItemLine.GetDropInfo` (`:78-109`).
        use UnlockState::*;
        // Tier 4: stack range + prettified rate.
        assert_eq!(drop_rate_text(CanShowDropsWithDropRates, 1.0, 1, 1), "100%");
        assert_eq!(
            drop_rate_text(CanShowDropsWithDropRates, 0.5, 2, 4),
            " (2-4) 50%"
        );
        assert_eq!(drop_rate_text(CanShowDropsWithDropRates, 0.25, 3, 3), " (3) 25%");
        // Tier 3: "???" and no stack range (`:105-109`).
        assert_eq!(drop_rate_text(CanShowDropsWithoutDropRates, 0.5, 2, 4), "???");
        // Below tier 3 the line is the locked icon, never text (`:36-55`).
        assert_eq!(drop_rate_text(CanShowStats, 1.0, 1, 1), "");
        assert_eq!(drop_rate_text(NotKnownAtAll, 1.0, 1, 1), "");
    }

    #[test]
    fn knockback_label_matches_the_four_way_gate() {
        // `NPCStatsReportInfoElement.cs:123`: > 0.8 High, > 0.4 Med,
        // <= 0 None, else Low; below tier 2 "???" (`:124-128`). The labels
        // are localized, so the assertions pin the GATE (four distinct
        // branches, the boundary values, the negative arm), not the words.
        let _guard = crate::lang::test_lock().lock().unwrap();
        assert_eq!(knockback_label(1.0, false), "???");
        assert_eq!(knockback_label(0.0, false), "???");
        let high = knockback_label(0.9, true);
        let med = knockback_label(0.5, true);
        let low = knockback_label(0.2, true);
        let none = knockback_label(0.0, true);
        let neg = knockback_label(-0.5, true);
        assert!(!high.is_empty() && !med.is_empty() && !low.is_empty() && !none.is_empty());
        assert_ne!(high, med);
        assert_ne!(med, low);
        assert_ne!(low, none);
        assert_eq!(none, neg, "negative resist is the None arm (!(x > 0))");
        // The exact boundaries: 0.8 is NOT > 0.8 (Med), 0.4 is NOT > 0.4 (Low).
        assert_eq!(knockback_label(0.8, true), med);
        assert_eq!(knockback_label(0.4, true), low);
        // The keys are the C#'s four (resolvable or the key placeholder).
        assert!(high == crate::lang::text("BestiaryInfo.KnockbackHigh"));
        assert!(none == crate::lang::text("BestiaryInfo.KnockbackNone"));
    }

    #[test]
    fn flavor_text_gates_at_tier_2_and_the_language_key() {
        // `FlavorTextBestiaryInfoElement.cs:20-23`: null below CanShowStats.
        let _guard = crate::lang::test_lock().lock().unwrap();
        let entry = &crate::bestiary::ENTRIES[0];
        assert!(flavor_text(entry, UnlockState::NotKnownAtAll).is_none());
        assert!(flavor_text(entry, UnlockState::CanShowPortraitOnly).is_none());
        // From tier 2: Some iff Language.Exists("Bestiary_FlavorText.npc_<id>")
        // (`BestiaryEntry.cs:46-50`) — without the resource files the manager
        // has no keys and the element is absent, exactly like a culture
        // missing the row.
        let got = flavor_text(entry, UnlockState::CanShowStats);
        let key = format!("Bestiary_FlavorText.npc_{}", entry.credit_id);
        assert_eq!(got.is_some(), crate::lang::exists(&key));
    }

    #[test]
    fn prettify_percent_display_trims_like_utils() {
        // `Utils.PrettifyPercentDisplay` (`Utils.cs:1105-1109`) — the
        // TrimStart('0', ' ') quirk is kept 1:1: a sub-1 % value loses its
        // leading zero (".01%"), and 0 trims to the bare "%".
        assert_eq!(prettify_percent_display(1.0), "100%");
        assert_eq!(prettify_percent_display(0.5), "50%");
        assert_eq!(prettify_percent_display(0.1429), "14.29%");
        assert_eq!(prettify_percent_display(0.0001), ".01%");
        assert_eq!(prettify_percent_display(0.0), "%");
    }

    // ---- W4-e23 remainder: the 2x2 stat grid + coin row. Written in
    // code-only mode — NOT RUN here.

    #[test]
    fn the_stat_grid_lays_out_the_two_by_two_lattice() {
        // `NPCStatsReportInfoElement.cs:69-96`: num=99 column pitch,
        // num2=35 row pitch, num3=3 left, num4=0 top. Append order:
        // HP (3,0), Attack (3,35), Defense (102,35), Knockback (102,0).
        let rects = stats_grid_rects([100.0, 50.0]);
        assert_eq!(rects[0], [103.0, 50.0, 96.0, 31.0]); // HP
        assert_eq!(rects[1], [103.0, 85.0, 96.0, 31.0]); // Attack
        assert_eq!(rects[2], [202.0, 85.0, 96.0, 31.0]); // Defense
        assert_eq!(rects[3], [202.0, 50.0, 96.0, 31.0]); // Knockback
        assert_eq!(STAT_ICON_SIZE, [96.0, 31.0]);
    }

    #[test]
    fn the_stat_value_rides_right_aligned_and_two_up() {
        // `:129-160`: HAlign 1, VAlign 0.5, Left -10, scale 1; the UIText
        // box is text_w x 16, DrawSelf rides Y -= 2*scale (`UIText.cs:138`).
        // A 40-wide text in the HP icon at (103, 50): x = 103+96-40-10 = 149,
        // y = 50 + (31-16)/2 - 2 = 55.5.
        let icon = [103.0, 50.0, 96.0, 31.0];
        assert_eq!(stat_value_pos(icon, 40.0, 1.0, -10.0), [149.0, 55.5]);
        // The coin texts (`:215-244`): Left -7, scale 0.85 — the box is
        // 16*0.85 = 13.6 tall, Y -= 1.7. A 20-wide text in a 47x31 coin at
        // (103, 128): x = 103+47-20-7 = 123, y = 128+(31-13.6)/2-1.7 = 135.
        let coin = [103.0, 128.0, 47.0, 31.0];
        assert_eq!(stat_value_pos(coin, 20.0, 0.85, -7.0), [123.0, 135.0]);
    }

    #[test]
    fn the_coin_strings_split_and_dash_like_the_source() {
        // `:99-119`: (int)value split into p/g/s/c, clamped 999/99/99/99,
        // a denomination whose division is < 1 prints "-".
        // 1 gold 23 silver 45 copper = 12345.
        assert_eq!(coin_strings(12345.0, true), ["-", "1", "23", "45"]);
        // 1 platinum 2 gold 3 silver 4 copper = 1020304.
        assert_eq!(coin_strings(1020304.0, true), ["1", "2", "3", "4"]);
        // 50 copper only.
        assert_eq!(coin_strings(50.0, true), ["-", "-", "-", "50"]);
        // Zero: every denomination dashes.
        assert_eq!(coin_strings(0.0, true), ["-", "-", "-", "-"]);
        // The platinum clamp: 1000 platinum reads 999 (`:100`).
        assert_eq!(coin_strings(1_000_000_000.0, true)[0], "999");
        // Below tier 2 all four become "?" (`:126`).
        assert_eq!(coin_strings(12345.0, false), ["?", "?", "?", "?"]);
    }

    #[test]
    fn the_coin_row_lays_out_the_four_denominations() {
        // `:176-198`: num9=3 left, num10=78 top, num11=49 pitch — the
        // silver/copper columns take the extra +1 (`:192`/`:197`).
        let rects = coin_row_rects([100.0, 50.0]);
        assert_eq!(rects[0], [103.0, 128.0, 47.0, 31.0]); // platinum
        assert_eq!(rects[1], [152.0, 128.0, 47.0, 31.0]); // gold
        assert_eq!(rects[2], [202.0, 128.0, 47.0, 31.0]); // silver (+1)
        assert_eq!(rects[3], [251.0, 128.0, 47.0, 31.0]); // copper (+1)
        assert_eq!(COIN_ICON_SIZE, [47.0, 31.0]);
    }

    #[test]
    fn the_stats_block_height_takes_the_coin_row() {
        // `:165`/`:249-251`: num8=66, +4 separator +34 coins when value>0,
        // Height = num8+4 — 70 without coins, 108 with.
        assert_eq!(stats_block_height(false), 70.0);
        assert_eq!(stats_block_height(true), 108.0);
    }

    #[test]
    fn the_kill_counter_panel_is_right_aligned_and_30_tall() {
        // `NPCKillCounterInfoElement.cs:54-65`: Width (-8 + -3, 1) = the
        // content width - 11, Height 30, Left -8, HAlign 1. A 206-wide
        // content box at x 100: the panel is 195 wide, its right edge at
        // 100+206-8 = 298, so x = 103.
        let content = [100.0, 50.0, 206.0, 300.0];
        assert_eq!(kill_counter_panel_rect(content, 400.0), [103.0, 400.0, 195.0, 30.0]);
    }

    // ---- W4-e23 remainder: the info-page scrollbar thumb. Written in
    // code-only mode — NOT RUN here.

    #[test]
    fn the_row_layout_stacks_with_list_padding() {
        // `UIList.RecalculateChildren` (`UIList.cs:116-128`): each row at
        // the running height, ListPadding between rows; a single row takes
        // no padding (`:122`). The total is GetTotalHeight (`:63-66`).
        let (tops, total) = info_page_row_layout(&[24.0, 112.0, 70.0], 4.0);
        assert_eq!(tops, vec![0.0, 28.0, 144.0]);
        assert_eq!(total, 24.0 + 4.0 + 112.0 + 4.0 + 70.0 + 4.0);
        // One row: no trailing padding.
        let (tops, total) = info_page_row_layout(&[24.0], 4.0);
        assert_eq!(tops, vec![0.0]);
        assert_eq!(total, 24.0);
        // Empty list: zero height.
        assert_eq!(info_page_row_layout(&[], 4.0), (vec![], 0.0));
    }

    #[test]
    fn the_view_height_drops_the_list_padding() {
        // The list's (2, 4, 2, 4) padding (`UIBestiaryEntryInfoPage.cs:40-42`):
        // the view height is the page height - 8.
        let page = [100.0, 50.0, 230.0, 400.0];
        assert_eq!(info_page_view_height(page), 392.0);
    }

    #[test]
    fn set_view_clamps_like_the_scrollbar() {
        // `UIScrollbar.SetView` (`UIScrollbar.cs:72-78`): the view clamps
        // into [0, max], the position into [0, max - view].
        assert_eq!(info_page_set_view(500.0, 300.0, 100.0), (300.0, 0.0));
        assert_eq!(info_page_set_view(100.0, 300.0, 500.0), (100.0, 200.0));
        assert_eq!(info_page_set_view(100.0, 300.0, -5.0), (100.0, 0.0));
        // A view taller than the content pins to the content, position 0.
        assert_eq!(info_page_set_view(400.0, 300.0, 50.0), (300.0, 0.0));
    }

    #[test]
    fn the_scrollbar_press_grabs_the_handle_or_jumps_the_track() {
        // `UIScrollbar.LeftMouseDown` (`UIScrollbar.cs:126-143`). The bar
        // at (304, 60) 20x380: inner y 65, inner h 370. With view 100,
        // max 1000, pos 0 the handle sits at y 65-3 = 62, h 37+7 = 44.
        let bar = [304.0, 60.0, 20.0, 380.0];
        let handle = super::super::panel::scrollbar_handle_at(bar, 100.0, 1000.0, 0.0);
        // A press inside the handle grabs it: position unchanged, the drag
        // offset is mouse.y - handle.y.
        let grab = info_page_scrollbar_press(bar, 100.0, 1000.0, 0.0, Vec2::new(310.0, handle[1] + 10.0));
        assert_eq!(grab, Some((0.0, Some(10.0))));
        // A press on the track (below the handle) jumps the view, no latch.
        let jump = info_page_scrollbar_press(bar, 100.0, 1000.0, 0.0, Vec2::new(310.0, 300.0));
        assert!(jump.is_some());
        assert_eq!(jump.unwrap().1, None);
        // Off the bar: no press.
        assert_eq!(info_page_scrollbar_press(bar, 100.0, 1000.0, 0.0, Vec2::new(100.0, 300.0)), None);
    }

    // ---- W4-e23 remainder: the biome portrait backdrops. Written in
    // code-only mode — NOT RUN here.

    #[test]
    fn the_backdrop_prefers_the_matched_provider() {
        // `NPCPortraitInfoElement.cs:43-57`: the first preference-matched
        // provider with an image wins. The Eater of Worlds head (6) carries
        // TheCorruption (tag 26, MapBG6) plain + preferred and
        // UndergroundCorruption (tag 27, MapBG23) plain — the preferred
        // TheCorruption wins both image and color (200, 200, 200).
        assert_eq!(portrait_backdrop(6), Some(("MapBG6", [200, 200, 200, 255])));
    }

    #[test]
    fn the_backdrop_falls_back_to_the_first_image_in_entry_order() {
        // `:58-70`: no matched image → the first provider with one. The
        // Guide (22) carries only Surface (tag 0, MapBG1, no color) → White.
        assert_eq!(portrait_backdrop(22), Some(("MapBG1", [255, 255, 255, 255])));
        // The Demon Eye (2) carries only NightTime (tag 5, MapBG1, the
        // (35, 40, 40) night tint).
        assert_eq!(portrait_backdrop(2), Some(("MapBG1", [35, 40, 40, 255])));
        // The Hornet (48) carries UndergroundMushroom (tag 25, MapBG21).
        assert_eq!(portrait_backdrop(48), Some(("MapBG21", [255, 255, 255, 255])));
    }

    #[test]
    fn the_world_evil_preference_picks_the_corrupt_arm() {
        // `BestiaryPortraitBackgroundBasedOnWorldEvilProviderPreferenceInfoElement.cs:22-29`:
        // the Desert Djinn (533) / Tomb Crawler (529) wrap the corrupt +
        // crimson underground-desert providers; the port's
        // `WorldGen.crimson == false` default picks the corrupt arm
        // (tag 30, MapBG40) over the plain crimson row (tag 35, MapBG41).
        assert_eq!(portrait_backdrop(533), Some(("MapBG40", [255, 255, 255, 255])));
        assert_eq!(portrait_backdrop(529), Some(("MapBG40", [255, 255, 255, 255])));
    }

    #[test]
    fn the_moon_lord_provider_paints_black() {
        // `MoonLordPortraitBackgroundProviderBestiaryInfoElement.cs:10-18`
        // (added at `BestiaryDatabaseNPCsPopulator.cs:639`): MapBG1 at
        // Color.Black. The Moon Lord (398) carries no tag rows, so its own
        // provider supplies both image and color.
        assert_eq!(portrait_backdrop(398), Some(("MapBG1", [0, 0, 0, 255])));
    }

    #[test]
    fn an_entry_without_providers_draws_no_backdrop() {
        // An id the populator tags with no background provider (the Rain
        // overlay element carries no image) — the pick is None and the
        // portrait rests on Slot_Back alone. The Goldfish (55) carries
        // Surface, so it DOES draw; a tagless id draws nothing.
        assert_eq!(portrait_backdrop(55), Some(("MapBG1", [255, 255, 255, 255])));
        // The Moon Lord's tagless roster neighbours: an id with no tag row
        // and no own provider (e.g. 999, outside the catalog) → None.
        assert_eq!(portrait_backdrop(999), None);
    }
}

/// `Utils.PrettifyPercentDisplay` (`Utils.cs:1105-1109`): the percent string
/// with the trailing zeros trimmed — "14.29 %" → "14.29%", "100%" for the
/// full rate, and the four-digit format under 0.1 %.
fn prettify_percent_display(rate: f32) -> String {
    if rate >= 1.0 {
        return "100%".to_string();
    }
    if rate < 0.001 {
        // The C#'s "P4" format (`:93-96`).
        let s = format!("{:.4}%", rate * 100.0);
        return trim_percent(&s);
    }
    let s = format!("{:.2}%", rate * 100.0);
    trim_percent(&s)
}

/// The trailing-zero trim of `PrettifyPercentDisplay`: `TrimEnd('0', '%', ' ')`
/// then `TrimEnd('.', ' ')` then `TrimStart('0', ' ')`.
fn trim_percent(s: &str) -> String {
    let trimmed = s
        .trim_end_matches(['0', '%', ' '])
        .trim_end_matches(['.', ' ']);
    format!("{}%", trimmed.trim_start_matches(['0', ' ']))
}
