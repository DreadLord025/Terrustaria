//! The three Workshop list screens (`GameContent/UI/States/`):
//! `UIWorkshopSelectResourcePackToPublish` (:17-235, W4-d84 row 2),
//! `UIWorkshopSelectWorldToPublish` (:16-233, row 3) and
//! `UIWorkshopWorldImport` (:18-251, row 4). They share one chrome — the
//! 650-max root at Top 220, the `(-110, 1)` container panel, the `UIList`
//! with the `Recalculate` scrollbar-attach branch, the centred title and
//! Back panels (`workshop_kit`'s geometry helpers) — and differ only in
//! their rows and their `OnActivate` populate.
//!
//! Deviations (docs/ANALYSIS.md "The Workshop/ResourcePack screens"):
//! - The Publish tail is a notice: `WorkshopPublishInfoStateForResourcePack`
//!   / `ForWorld` (the Steam publish form,
//!   `UIWorkshopPublishResourcePackListItem.cs:129`) are out of the d84
//!   range; the click sets `front_hint` (the world-select rejection's
//!   convention).
//! - No Steam: the import screen's subscribed-world list is always empty
//!   (`UpdateWorkshopWorldList`, `UIWorkshopWorldImport.cs:142-153`), and
//!   the pack rows' `canPublish` is always true (no zip packs), so the
//!   grey row colours and the zip rejection menu stay unreachable —
//!   ported for the record, exercised by tests only.
//! - The gamepad snap points (`SetupGamepadPoints`) are out of scope (§J.3).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::resource_pack::ResourcePackList;
use crate::ui::text::{self, font_width};
use crate::ui::uistate;
use crate::ui::{color, framework, panel, save_list, workshop_kit as kit};

/// The pack-publish screen's own state — `_entries`
/// (`UIWorkshopSelectResourcePackToPublish.cs:31`, the publishable list),
/// the `UIScrollbar.ViewPosition` and the `_menuToGoBackTo` target (`:29`).
/// The hub is the only opener (`Click_OpenResourcePackPublishMenu`,
/// `UIWorkshopHub.cs:156-160`), so `back` parks it.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PublishPacksState {
    /// `_entries` — `AssetInitializer.CreatePublishableResourcePacksList`
    /// filtered to `Branding == None` and ordered by `IsCompressed`
    /// (`PopulateEntries`, `:137-151`).
    pub packs: ResourcePackList,
    /// The `_entryList` row order (`PopulateEntries`, `:146-150`) — store
    /// indices, re-derived by [`Self::populate`].
    pub(crate) view: Vec<usize>,
    /// The list's `UIScrollbar.ViewPosition`.
    pub scroll: f32,
    /// `_menuToGoBackTo` (`:29`) — `HandleBackButtonUsage` (`:109-113`)
    /// restores it verbatim.
    pub back: Option<Box<crate::app::flow::FrontScreen>>,
}

impl PublishPacksState {
    /// `PopulateEntries` (`:137-151`): the publishable list (no saved
    /// state — `ResourcePackList.Publishable`, `IO/ResourcePackList.cs:68-79`),
    /// `where Branding == None` (always true without Steam) `orderby
    /// IsCompressed` (a stable sort; all false in the port, so the
    /// `AllPacks` name/version/file order survives).
    pub(crate) fn populate(&mut self) {
        let mut view = self.packs.all_packs();
        view.sort_by_key(|&i| self.packs.packs[i].is_compressed);
        // The view order IS the row order; the store keeps discovery order.
        self.view = view;
    }

    /// The row order `PopulateEntries` produced (store indices).
    pub(crate) fn rows(&self) -> &[usize] {
        &self.view
    }
}

/// Every static rect the three list screens share (`OnInitialize`,
/// `UIWorkshopSelectResourcePackToPublish.cs:40-82` and twins). `attached`
/// carries the `Recalculate` branch (`:84-102`): the list width drops 25
/// while the scrollbar is a panel child.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ListLayout {
    /// The 80%-wide, 650-max root at Top 220 (`:42-47`).
    pub root: [f32; 4],
    /// The container panel, `Height (-110, 1)` (`:48-51`).
    pub panel: [f32; 4],
    /// The `UIList` — `Width (0, 1)` detached / `(-25, 1)` attached
    /// (`:55`, `:92/:98`), `Height (0, 1)` of the panel inner.
    pub list: [f32; 4],
    /// The scrollbar attach rect (`:59-62`): `Width 20`, `HAlign 1`.
    pub scrollbar: [f32; 4],
    /// The centred Back panel (`:70-75`).
    pub back: [f32; 4],
    /// `Recalculate`'s `_isScrollbarAttached` (`:88-99`).
    pub attached: bool,
}

/// The shared chrome for `rows` rows of `row_h` px cards at the 5 px
/// `ListPadding` pitch (`UIList.cs:120-128`). The attach test is
/// `UIList.UpdateScrollbar`'s `SetView` + `CanScroll`
/// (`kit::list_scrollbar_attaches`).
pub(crate) fn list_layout(screen: Vec2, rows: usize, row_h: f32) -> ListLayout {
    let root = kit::workshop_root(screen);
    let panel = kit::workshop_panel(root);
    let panel_inner = uistate::inner(panel, [12.0; 4]);
    let content_h = content_height(rows, row_h);
    let attached = kit::list_scrollbar_attaches(panel_inner[3], content_h);
    // `:55` / `:98` — the list keeps the full inner width detached, drops
    // 25 attached; its height is the inner height either way (`:56`).
    let list = [
        panel_inner[0],
        panel_inner[1],
        panel_inner[2] - if attached { 25.0 } else { 0.0 },
        panel_inner[3],
    ];
    let scrollbar = kit::workshop_scrollbar(panel_inner);
    let back = kit::workshop_back(root);
    ListLayout { root, panel, list, scrollbar, back, attached }
}

/// The `UIList` content height for `rows` cards of `row_h` px
/// (`UIList.cs:120-128`): one 5 px `ListPadding` per row, a single row
/// drops it. 82 px publish rows (`kit::publish_content_height`) and 96 px
/// import rows are the two callers.
pub(crate) fn content_height(rows: usize, row_h: f32) -> f32 {
    match rows {
        0 => 0.0,
        1 => row_h,
        n => n as f32 * (row_h + 5.0),
    }
}

/// The element ids the pointer machine needs — the publish/import rows'
/// `OnLeftDoubleClick` subscriptions (`UIWorkshopPublishResourcePackListItem.cs:56/:82`,
/// `UIWorkshopPublishWorldListItem.cs:35/:42`,
/// `UIWorkshopImportWorldListItem.cs:39/:46`) fire on the same element
/// twice inside 500 ms, the `select_hit` pattern.
pub(crate) const BACK: u32 = 1;
/// Row `i`'s body and action button occupy `ROW_BASE + i*2 .. +2`.
pub(crate) const ROW_BASE: u32 = 2;
pub(crate) const PART_BODY: u32 = 0;
pub(crate) const PART_ACTION: u32 = 1;

pub(crate) const fn row_part(row: usize, part: u32) -> u32 {
    ROW_BASE + row as u32 * 2 + part
}

/// What a click on a list screen hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Click {
    None,
    /// `GoBackClick` → `HandleBackButtonUsage` (`:104-113` and twins).
    Back,
    /// The Publish button (single click) or the card/icon double-click
    /// (`PublishButtonClick_ImportResourcePackToLocalFiles`, `:125-131`;
    /// the world twin `:73-79`).
    Publish(usize),
    /// The import row's play button / double-click
    /// (`ImportButtonClick_ImportWorldToLocalFiles`,
    /// `UIWorkshopImportWorldListItem.cs:96-105`).
    Import(usize),
}

/// The flat element list in the C# append order: the rows (body first,
/// then the action button — the button is the card's appended child, so
/// the reverse walk tests it first), then Back (appended to the root
/// after the panel, `:79-81`). `action_rect` picks the row's button rect.
fn elements(
    l: &ListLayout,
    rows: usize,
    row_h: f32,
    scroll: f32,
    action_rect: impl Fn([f32; 4], usize) -> [f32; 4],
) -> Vec<framework::Element> {
    let mut out = Vec::with_capacity(rows * 2 + 1);
    // The rows ride the translated inner list (`UIList.cs:181`); the
    // viewport stays the `UIList`'s own rect (`UIList.cs:59`).
    let row_rect = [l.list[0], l.list[1] - scroll, l.list[2], l.list[3]];
    for i in 0..rows {
        let card = [row_rect[0], row_rect[1] + i as f32 * (row_h + 5.0), row_rect[2], row_h];
        let body = row_part(i, PART_BODY);
        out.push(framework::Element::row(card, body, l.list));
        out.push(framework::Element::row_child(
            action_rect(row_rect, i),
            row_part(i, PART_ACTION),
            body,
            l.list,
        ));
    }
    out.push(framework::Element::hit(l.back, BACK));
    out
}

/// The element under the pointer — `App::front_hit`'s feed for the
/// pointer machine (the double-click identity).
pub(crate) fn hit_element(
    l: &ListLayout,
    rows: usize,
    row_h: f32,
    scroll: f32,
    action_rect: impl Fn([f32; 4], usize) -> [f32; 4],
    mouse: Vec2,
) -> Option<u32> {
    framework::get_element_at(&elements(l, rows, row_h, scroll, action_rect), mouse)
}

/// The click table over the scrolled list. `double_click` is the pointer
/// machine's `DoubleClick` for this press — the card body's
/// `OnLeftDoubleClick` publishes/imports exactly like the button.
pub(crate) fn click_at(
    l: &ListLayout,
    rows: usize,
    row_h: f32,
    scroll: f32,
    action_rect: impl Fn([f32; 4], usize) -> [f32; 4],
    mouse: Vec2,
    double_click: bool,
    import: bool,
) -> Click {
    let Some(id) = hit_element(l, rows, row_h, scroll, action_rect, mouse) else {
        return Click::None;
    };
    if id == BACK {
        return Click::Back;
    }
    let row = ((id - ROW_BASE) / 2) as usize;
    let part = (id - ROW_BASE) % 2;
    match part {
        PART_ACTION => {
            if import { Click::Import(row) } else { Click::Publish(row) }
        }
        // The body: only the double-click subscription fires
        // (`base.OnLeftDoubleClick`, `:82` and twins).
        _ if double_click => {
            if import { Click::Import(row) } else { Click::Publish(row) }
        }
        _ => Click::None,
    }
}

// --- The publish rows (82 px) ------------------------------------------------

/// One publish row's geometry — `UIWorkshopPublishResourcePackListItem`
/// (`:41-101`, `DrawSelf` `:187-215`) and its world twin
/// (`UIWorkshopPublishWorldListItem.cs:25-59`, `:107-139`). The card is
/// 82 px, `SetPadding(6)`; the icon block sits `Left 4`, `VAlign 0.5`;
/// the Publish button `HAlign 1`, `VAlign 1`; the two info plates split
/// the space between the icon and the button.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PublishRow {
    /// The 82 px card panel (`:97-99`).
    pub card: [f32; 4],
    /// The icon element — 72x72 for packs (`:49-55`), the 60x58 world
    /// icon for worlds (`UIWorkshopPublishWorldListItem.cs:32-34`).
    pub icon: [f32; 4],
    /// The pack icon's inner `UIImage` — `(-6, 1)` centred, `ScaleToFit`
    /// (`:57-66`); `None` on world rows.
    pub pack_icon: Option<[f32; 4]>,
    /// The pack icon's `Achievement_Borders` frame, centred
    /// (`:67-72`); `None` on world rows.
    pub icon_border: Option<[f32; 4]>,
    /// The Publish button — `HAlign 1`, `VAlign 1` of the inner
    /// (`:79-80`), sized by `kit::publish_button_size`.
    pub publish: [f32; 4],
    /// The name position — packs `(icon right + 8, inner.Y + 3)`
    /// (`:193`), worlds `(icon right + 6, inner.Y + 3)` (`:113`).
    pub name_pos: [f32; 2],
    /// The first info plate (author / difficulty) — `num2` wide at the
    /// button's height, bottom-aligned (`:194-198` / `:114-118`).
    pub plate1: [f32; 4],
    /// The second plate (version / size) — `+ num2 + 5` (`:205-207` /
    /// `:126-132`); the world size plate grows 40 px off-English.
    pub plate2: [f32; 4],
}

/// Row `index`'s publish geometry. `icon_w`/`icon_h` are 72/72 for packs
/// and 60/58 for worlds; `btn` is the measured Publish button size;
/// `size_plate_extra` is the non-English `+40f`
/// (`UIWorkshopPublishWorldListItem.cs:128-131`), 0 for packs.
pub(crate) fn publish_row(
    row_rect: [f32; 4],
    index: usize,
    icon_w: f32,
    icon_h: f32,
    btn: [f32; 2],
    size_plate_extra: f32,
) -> PublishRow {
    let card = [row_rect[0], row_rect[1] + index as f32 * 87.0, row_rect[2], 82.0];
    let inner = uistate::inner(card, [6.0; 4]);
    // `Left 4`, `VAlign 0.5` of the inner (`:53-54` / `:33-34`).
    let icon = [
        inner[0] + 4.0,
        inner[1] + (inner[3] - icon_h) * 0.5,
        icon_w,
        icon_h,
    ];
    let pack_icon = if icon_w == 72.0 {
        // `(-6, 1)` centred in the 72x72 area (`:59-62`).
        Some([icon[0] + 3.0, icon[1] + 3.0, 66.0, 66.0])
    } else {
        None
    };
    let icon_border = if icon_w == 72.0 {
        // The 72x72 frame centred in the area (`:69-70`).
        Some(icon)
    } else {
        None
    };
    let publish = [
        inner[0] + inner[2] - btn[0],
        inner[1] + inner[3] - btn[1],
        btn[0],
        btn[1],
    ];
    let icon_right = icon[0] + icon[2];
    let name_gap = if icon_w == 72.0 { 8.0 } else { 6.0 };
    let name_pos = [icon_right + name_gap, inner[1] + 3.0];
    // `num2 = (innerW - 22 - iconW - buttonW) / 2` (`:194` / `:114`).
    let num2 = (inner[2] - 22.0 - icon_w - btn[0]) / 2.0;
    let plate_y = inner[1] + inner[3] - btn[1];
    let plate1 = [icon_right + name_gap, plate_y, num2, btn[1]];
    let plate2 = [plate1[0] + num2 + 5.0, plate_y, num2 + size_plate_extra, btn[1]];
    PublishRow { card, icon, pack_icon, icon_border, publish, name_pos, plate1, plate2 }
}

/// The publish rows' hover colours (`SetColorsToHovered`/`ToNotHovered`,
/// `UIWorkshopPublishResourcePackListItem.cs:103-123`; the world twin has
/// no grey arm, `:61-71`). `can_publish` is `!IsCompressed` — always true
/// in the port (no zips), so the grey pair is unreachable live; ported
/// for the record. XNA byte truncation throughout.
pub(crate) fn publish_row_colors(hover: bool, can_publish: bool) -> ([u8; 4], [u8; 4]) {
    if can_publish {
        if hover {
            ([73, 94, 171, 255], [89, 116, 213, 255]) // `:105-106`
        } else {
            (
                color::mul([63, 82, 151, 255], 0.7), // `:116`
                color::mul([89, 116, 213, 255], 0.7), // `:117`
            )
        }
    } else if hover {
        // `:109-110` — border (150,150,150), bg Lerp(hover, (120,120,120), .5).
        (
            kit::lerp_bytes([73, 94, 171, 255], [120, 120, 120, 255], 0.5),
            [150, 150, 150, 255],
        )
    } else {
        // `:120-121` — border (127,127,127)*0.7, bg Lerp(rest, (80,80,80), .5)*0.7.
        (
            color::mul(kit::lerp_bytes([63, 82, 151, 255], [80, 80, 80, 255], 0.5), 0.7),
            color::mul([127, 127, 127, 255], 0.7),
        )
    }
}

/// The Publish button size without a renderer — the click table's twin
/// of `kit::publish_button_size` (the bitmap fallback estimates 8 px per
/// char, the `pack_info_desc_height` convention).
pub(crate) fn publish_button_size_headless(lookups: &assets::Lookups) -> [f32; 2] {
    let title = crate::lang::text("Workshop.Publish");
    let title_w = match &lookups.ui_font {
        Some(f) => font_width(f, &title, 1.0),
        None => title.chars().count() as f32 * 8.0,
    };
    match lookups.workshop_publish_icon {
        Some(icon) => [
            title_w + icon.width as f32 + 26.0,
            16.0f32.max(icon.height as f32) + 16.0,
        ],
        None => [44.0, 34.0],
    }
}

/// The pack row's Publish button rect for the click table / hit ids.
pub(crate) fn pack_action_rect(lookups: &assets::Lookups, row_rect: [f32; 4], i: usize) -> [f32; 4] {
    publish_row(row_rect, i, 72.0, 72.0, publish_button_size_headless(lookups), 0.0).publish
}

// --- The pack-publish screen's draw ------------------------------------------

/// `UIWorkshopSelectResourcePackToPublish.Draw` (`:153-162` → `base.Draw`):
/// the shared chrome plus one `UIWorkshopPublishResourcePackListItem` per
/// publishable pack. `mouse` drives the row hover and the Back panel.
pub(crate) fn draw_publish_packs(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &PublishPacksState,
    screen: Vec2,
    mouse: Vec2,
) {
    let rows = state.rows();
    let l = list_layout(screen, rows.len(), 82.0);
    draw_chrome(r, lookups, &l, "UI.WorkshopSelectResourcePackToPublishMenuTitle", mouse);
    let btn = kit::publish_button_size(r, lookups);
    let row_rect = [l.list[0], l.list[1] - state.scroll, l.list[2], l.list[3]];
    for (i, &store) in rows.iter().enumerate() {
        let row = publish_row(row_rect, i, 72.0, 72.0, btn, 0.0);
        // The viewport filter (`UIList.cs:59`'s OverflowHidden).
        if row.card[1] + row.card[3] <= l.list[1] || row.card[1] >= l.list[1] + l.list[3] {
            continue;
        }
        let pack = &state.packs.packs[store];
        let hover = crate::ui::in_rect(mouse, row.card);
        // `canPublish = !entry.IsCompressed` (`:149`) — always true here.
        let (bg, border) = publish_row_colors(hover, !pack.is_compressed);
        panel::draw_panel(r, lookups, lookups.ui_panel_bg, row.card, color::tint(bg));
        panel::draw_panel(r, lookups, lookups.ui_panel_border, row.card, color::tint(border));
        // The icon block (`:57-74`): the fallback pack icon scaled into
        // the 66x66 inner box, the Achievement_Borders frame over it.
        if let (Some(icon), Some(rect)) = (lookups.default_pack_icon, row.pack_icon) {
            crate::ui::push_whole(r, lookups, &icon, rect, [1.0; 4]);
        }
        if let (Some(borders), Some(rect)) = (lookups.ach_borders, row.icon_border) {
            crate::ui::push_whole(r, lookups, &borders, rect, [1.0; 4]);
        }
        // The name (`:193`): White, `(icon right + 8, inner.Y + 3)`.
        text::border_string(r, lookups, row.name_pos[0], row.name_pos[1], [1.0; 4], &pack.name);
        // The two info plates (`:194-214`): author then version, centred
        // with the `+3` y offset, at the button's height.
        kit::draw_info_plate(r, lookups, row.plate1);
        let author = kit::author_line(pack);
        draw_plate_text(r, lookups, row.plate1, &author, [1.0; 4]);
        kit::draw_info_plate(r, lookups, row.plate2);
        let version = kit::version_line(pack);
        draw_plate_text(r, lookups, row.plate2, &version, [1.0; 4]);
        // The Publish button (`:78-84`), its own hover state.
        kit::draw_publish_button(r, lookups, row.publish, hover && crate::ui::in_rect(mouse, row.publish));
    }
    draw_list_scrollbar(r, lookups, &l, rows.len(), 82.0, state.scroll);
}

/// The shared chrome draw: the container panel, the centred title
/// (`:64-69`) and the Back panel (`:70-80`, `FadedMouseOver`/`Out`).
fn draw_chrome(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &ListLayout,
    title_key: &str,
    mouse: Vec2,
) {
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(kit::CLUSTER_PANEL_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.panel, color::tint(uistate::TEXT_PANEL_BORDER));
    let title = crate::lang::text(title_key);
    let title_w = match &lookups.death_font {
        Some(f) => font_width(f, &title, 1.0),
        None => r.text_width(&title, 1.0),
    };
    let title_rect = kit::workshop_title(l.root, title_w);
    uistate::draw_text_panel(
        r, lookups, title_rect,
        &uistate::TextPanelStyle {
            text_scale: 0.8,
            large: true,
            padding: 15.0,
            bg: kit::TITLE_BG,
            border: uistate::TEXT_PANEL_BORDER,
        },
        &title, title_w, false,
    );
    let back_text = crate::lang::text("UI.Back");
    let back_w = match &lookups.death_font {
        Some(f) => font_width(f, &back_text, 1.0),
        None => r.text_width(&back_text, 1.0),
    };
    uistate::draw_text_panel(
        r, lookups, l.back,
        &uistate::TextPanelStyle {
            text_scale: 0.7,
            large: true,
            padding: 12.0,
            bg: uistate::TEXT_PANEL_BG,
            border: uistate::TEXT_PANEL_BORDER,
        },
        &back_text, back_w, crate::ui::in_rect(mouse, l.back),
    );
}

/// The attached scrollbar (`Recalculate`, `:84-102` — drawn only while
/// attached; `AutoHide` stays false, `UIScrollbar.cs:29`).
fn draw_list_scrollbar(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    l: &ListLayout,
    rows: usize,
    row_h: f32,
    scroll: f32,
) {
    if l.attached {
        panel::draw_scrollbar(r, lookups, l.scrollbar, l.list[3], content_height(rows, row_h), scroll, false);
    }
}

/// One info plate's centred label — `vector + (plateW/2 - textW/2,
/// plateH/2 - textH/2 + 3)` (`UIWorkshopPublishResourcePackListItem.cs:204`,
/// the `MeasureString` Y is the Mouse font's line height).
fn draw_plate_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    plate: [f32; 4],
    label: &str,
    tint: [f32; 4],
) {
    let w = text::hud_text_width(r, lookups, label, 1.0);
    let h = text::line_height(lookups);
    text::border_string(
        r, lookups,
        plate[0] + plate[2] * 0.5 - w * 0.5,
        plate[1] + plate[3] * 0.5 - h * 0.5 + 3.0,
        tint, label,
    );
}

/// The non-English size plate's `+40f`
/// (`UIWorkshopPublishWorldListItem.cs:128-131`,
/// `UIWorkshopImportWorldListItem.cs:182-185`):
/// `!GameCulture.English.IsActive`.
pub(crate) fn size_plate_extra() -> f32 {
    if crate::lang::active_culture() == tr_core::localization::GameCulture::English {
        0.0
    } else {
        40.0
    }
}

/// `UIWorkshopSelectWorldToPublish.PopulateEntries` (`:134-149`):
/// `Main.LoadWorlds`, drop `!IsValid`, `orderby IsFavorite descending,
/// Name, GetFileName()` — NOT the world-select sort (no Journey key).
pub(crate) fn sort_publish_worlds(worlds: &mut [save_list::WorldEntry]) {
    worlds.sort_by(|a, b| {
        b.is_favorite
            .cmp(&a.is_favorite)
            .then_with(|| a.name.cmp(&b.name))
            .then_with(|| {
                let fa = a.path.as_deref().and_then(std::path::Path::file_name);
                let fb = b.path.as_deref().and_then(std::path::Path::file_name);
                fa.cmp(&fb)
            })
    });
}

// --- The world-publish screen (row 3) ----------------------------------------

/// `UIWorkshopSelectWorldToPublish`'s own state — the `_entryList` rows
/// (`PopulateEntries`, `:134-149`: `Main.LoadWorlds`, drop `!IsValid`,
/// the favourite/name/file sort), the list scroll and the
/// `_menuToGoBackTo` target (`:28`).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PublishWorldsState {
    /// The valid worlds in `PopulateEntries` order.
    pub worlds: Vec<save_list::WorldEntry>,
    /// The list's `UIScrollbar.ViewPosition`.
    pub scroll: f32,
    /// `_menuToGoBackTo` (`:28`) — the hub.
    pub back: Option<Box<crate::app::flow::FrontScreen>>,
}

/// The world row's Publish button rect for the click table / hit ids.
pub(crate) fn world_action_rect(lookups: &assets::Lookups, row_rect: [f32; 4], i: usize) -> [f32; 4] {
    publish_row(
        row_rect, i, 60.0, 58.0,
        publish_button_size_headless(lookups),
        size_plate_extra(),
    )
    .publish
}

/// `UIWorkshopSelectWorldToPublish.Draw` (`:151-160` → `base.Draw`): the
/// shared chrome plus one `UIWorkshopPublishWorldListItem` per valid
/// world (`:107-139`'s DrawSelf — the name, the difficulty/size plates,
/// the Publish button).
pub(crate) fn draw_publish_worlds(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &PublishWorldsState,
    screen: Vec2,
    mouse: Vec2,
) {
    let l = list_layout(screen, state.worlds.len(), 82.0);
    draw_chrome(r, lookups, &l, "UI.WorkshopSelectWorldToPublishMenuTitle", mouse);
    let btn = kit::publish_button_size(r, lookups);
    let extra = size_plate_extra();
    let row_rect = [l.list[0], l.list[1] - state.scroll, l.list[2], l.list[3]];
    for (i, entry) in state.worlds.iter().enumerate() {
        let row = publish_row(row_rect, i, 60.0, 58.0, btn, extra);
        if row.card[1] + row.card[3] <= l.list[1] || row.card[1] >= l.list[1] + l.list[3] {
            continue;
        }
        let hover = crate::ui::in_rect(mouse, row.card);
        // The world twin has no grey arm (`:61-71`) — every valid row
        // publishes.
        let (bg, border) = publish_row_colors(hover, true);
        panel::draw_panel(r, lookups, lookups.ui_panel_bg, row.card, color::tint(bg));
        panel::draw_panel(r, lookups, lookups.ui_panel_border, row.card, color::tint(border));
        // The 60x58 world icon (`GetIconElement`, `AWorldListItem.cs:181-184`)
        // — the fixed IconCrimson sheet (the world-select deviation: no
        // evil/hardmode data, no cycling).
        if let Some(sheet) = lookups.ui_world_icon {
            crate::ui::push_whole(r, lookups, &sheet, row.icon, [1.0; 4]);
        }
        // The name (`:113`): White (the rows are all valid — the Gray arm
        // is unreachable after PopulateEntries' RemoveAll), cropped at
        // 494 px (`GetWorldName(allowCropping: true)`, `WorldFileData.cs:
        // 109-121`) — the port's names are short, the crop is a no-op.
        text::border_string(r, lookups, row.name_pos[0], row.name_pos[1], [1.0; 4], &entry.name);
        // The difficulty plate (`:119-125`): GetDifficulty's label+colour.
        kit::draw_info_plate(r, lookups, row.plate1);
        let (diff_text, diff_color) = crate::ui::world_select::difficulty_text(entry.difficulty);
        draw_plate_text(r, lookups, row.plate1, &diff_text, color::tint(diff_color));
        // The size plate (`:126-137`): "{0} World", White.
        kit::draw_info_plate(r, lookups, row.plate2);
        let size_text = crate::lang::text_args(
            "UI.WorldSizeFormat",
            &[crate::ui::world_select::size_name(entry.size_id)],
        );
        draw_plate_text(r, lookups, row.plate2, &size_text, [1.0; 4]);
        kit::draw_publish_button(r, lookups, row.publish, hover && crate::ui::in_rect(mouse, row.publish));
    }
    draw_list_scrollbar(r, lookups, &l, state.worlds.len(), 82.0, state.scroll);
}

// --- The world-import screen (row 4) -----------------------------------------

/// `UIWorkshopWorldImport`'s own state — the `_worldList` rows
/// (`UpdateWorldsList`, `UIWorkshopWorldImport.cs:155-166`: the static
/// `WorkshopWorldList` sorted favourite/name/file), the list scroll and the
/// `_uiStateToGoBackTo` target (`:32`). Without Steam
/// `UpdateWorkshopWorldList` (`:142-153`) returns early, so the list is
/// always empty live — the rows are ported for the record.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ImportWorldsState {
    /// The subscribed worlds in `UpdateWorldsList` order (always empty in
    /// the port — no Steam).
    pub worlds: Vec<save_list::WorldEntry>,
    /// The list's `UIScrollbar.ViewPosition`.
    pub scroll: f32,
    /// `_uiStateToGoBackTo` (`:32`) — the hub.
    pub back: Option<Box<crate::app::flow::FrontScreen>>,
}

/// One import row's geometry — `UIWorkshopImportWorldListItem`
/// (`UIWorkshopImportWorldListItem.cs:30-71`, `DrawSelf` `:151-198`). The
/// card is 96 px, `SetPadding(6)`; the 60x58 world icon sits `Left 4` at
/// the inner TOP (no VAlign — the publish twin's 0.5 is absent); the play
/// `UIImageButton` `VAlign 1`, `Left 4`; the plates ride a 3-slice
/// horizontal `DrawPanel` (`:144-149`) at the sheet's own height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ImportRow {
    /// The 96 px card panel (`:68-70`).
    pub card: [f32; 4],
    /// The 60x58 world icon at `inner + (4, 0)` (`:37-38`).
    pub icon: [f32; 4],
    /// The play button — `VAlign 1`, `Left 4` (`:42-44`), sheet-sized
    /// (`UIImageButton.cs:31-32`).
    pub play: [f32; 4],
    /// The hover label's draw position — `Left 28`, `VAlign 1`, `Top -3`
    /// (`:50-54`) minus the small-UIText `2*scale` shift (`UIText.cs:138`).
    pub label_pos: [f32; 2],
    /// The workshop badge — `(card.right - w - 3, card.y + 2)` (`:168`).
    pub badge: [f32; 4],
    /// The divider — `(icon right, inner.Y + 21)`, stretched to the card's
    /// right edge (`:169`).
    pub divider: [f32; 4],
    /// The difficulty plate — 100 wide at the inner-panel sheet height
    /// (`:170-172`).
    pub plate1: [f32; 4],
    /// The size plate — 150 wide (+40 off-English, `:181-185`).
    pub plate2: [f32; 4],
    /// The created plate — the rest of the inner width (`:192-193`).
    pub plate3: [f32; 4],
}

/// Row `index`'s import geometry. `btn` is the ButtonPlay sheet size
/// (22x22), `badge` the Extra_243 size, `divider_h`/`plate_h` the sheet
/// heights (4 / 27 fallbacks).
pub(crate) fn import_row(
    row_rect: [f32; 4],
    index: usize,
    btn: [f32; 2],
    badge: [f32; 2],
    divider_h: f32,
    plate_h: f32,
    size_plate_extra: f32,
) -> ImportRow {
    let card = [row_rect[0], row_rect[1] + index as f32 * 101.0, row_rect[2], 96.0];
    let inner = uistate::inner(card, [6.0; 4]);
    let icon = [inner[0] + 4.0, inner[1], 60.0, 58.0];
    let play = [inner[0] + 4.0, inner[1] + inner[3] - btn[1], btn[0], btn[1]];
    // The label element: VAlign 1 of its 16 px MinHeight, Top -3 (`:52-54`);
    // the small UIText draws 2 px above (`UIText.cs:138`).
    let label_pos = [inner[0] + 28.0, inner[1] + inner[3] - 16.0 - 3.0 - 2.0];
    let badge_rect = [
        card[0] + card[2] - badge[0] - 3.0,
        card[1] + 2.0,
        badge[0],
        badge[1],
    ];
    let icon_right = icon[0] + icon[2];
    let divider = [icon_right, inner[1] + 21.0, card[0] + card[2] - icon_right, divider_h];
    let plate_y = inner[1] + 29.0;
    let plate1 = [icon_right + 6.0, plate_y, 100.0, plate_h];
    let plate2 = [plate1[0] + 105.0, plate_y, 150.0 + size_plate_extra, plate_h];
    let plate3_x = plate2[0] + plate2[2] + 5.0;
    let plate3 = [plate3_x, plate_y, inner[0] + inner[2] - plate3_x, plate_h];
    ImportRow { card, icon, play, label_pos, badge: badge_rect, divider, plate1, plate2, plate3 }
}

/// The import row's play-button rect for the click table / hit ids — the
/// headless ButtonPlay size (the sheet, or the 22x22 ctor fallback).
pub(crate) fn import_action_rect(lookups: &assets::Lookups, row_rect: [f32; 4], i: usize) -> [f32; 4] {
    let btn = lookups.card_buttons[0]
        .map_or([22.0, 22.0], |s| [s.width as f32, s.height as f32]);
    import_row(row_rect, i, btn, [0.0, 0.0], 4.0, 27.0, size_plate_extra()).play
}

/// `UIWorkshopImportWorldListItem.DrawPanel` (`:144-149`): the
/// InnerPanelBackground's 3-slice horizontal strip — the left/right 8 px
/// columns at the sheet's full height, the middle 8 px column stretched to
/// `width - 16` (NOT the publish rows' 10-slice `DrawSplicedPanel`).
fn draw_import_plate(r: &mut SpriteRenderer, lookups: &assets::Lookups, rect: [f32; 4]) {
    let Some(sheet) = lookups.ui_inner_panel else { return };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let h = sheet.height as f32;
    let mut piece = |dx: f32, dw: f32, sx: f32| {
        if dw <= 0.0 {
            return;
        }
        let uv = [
            (sheet.x as f32 + sx) / aw,
            sheet.y as f32 / ah,
            (sheet.x as f32 + sx + 8.0) / aw,
            (sheet.y as f32 + h) / ah,
        ];
        r.push_ui_sprite([rect[0] + dx, rect[1]], [dw, h], uv, [1.0; 4]);
    };
    piece(0.0, 8.0, 0.0);
    piece(8.0, rect[2] - 16.0, 8.0);
    piece(rect[2] - 8.0, 8.0, 16.0);
}

/// The import plate's centred label — `vector + (plateW/2 - textW/2, 3)`
/// (`:177/:189/:196`: the y offset is a flat +3, not the publish plates'
/// height-centred form).
fn draw_import_plate_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    plate: [f32; 4],
    label: &str,
    tint: [f32; 4],
) {
    let w = text::hud_text_width(r, lookups, label, 1.0);
    text::border_string(r, lookups, plate[0] + plate[2] * 0.5 - w * 0.5, plate[1] + 3.0, tint, label);
}

/// `UIWorkshopWorldImport.Draw` (`:168-177` → `base.Draw`): the shared
/// chrome plus one `UIWorkshopImportWorldListItem` per subscribed world
/// (always none without Steam — the rows are ported for the record).
pub(crate) fn draw_import_worlds(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &ImportWorldsState,
    screen: Vec2,
    mouse: Vec2,
) {
    let l = list_layout(screen, state.worlds.len(), 96.0);
    draw_chrome(r, lookups, &l, "UI.WorkshopImportWorld", mouse);
    let btn = lookups.card_buttons[0]
        .map_or([22.0, 22.0], |s| [s.width as f32, s.height as f32]);
    let badge = lookups.workshop_icon
        .map_or([0.0, 0.0], |s| [s.width as f32, s.height as f32]);
    let divider_h = lookups.ui_divider.map_or(4.0, |s| s.height as f32);
    let plate_h = lookups.ui_inner_panel.map_or(27.0, |s| s.height as f32);
    let extra = size_plate_extra();
    let row_rect = [l.list[0], l.list[1] - state.scroll, l.list[2], l.list[3]];
    for (i, entry) in state.worlds.iter().enumerate() {
        let row = import_row(row_rect, i, btn, badge, divider_h, plate_h, extra);
        if row.card[1] + row.card[3] <= l.list[1] || row.card[1] >= l.list[1] + l.list[3] {
            continue;
        }
        let hover = crate::ui::in_rect(mouse, row.card);
        // The import twin has no grey arm either (`:74-84`).
        let (bg, border) = publish_row_colors(hover, true);
        panel::draw_panel(r, lookups, lookups.ui_panel_bg, row.card, color::tint(bg));
        panel::draw_panel(r, lookups, lookups.ui_panel_border, row.card, color::tint(border));
        // The 60x58 world icon at the inner top (no VAlign).
        if let Some(sheet) = lookups.ui_world_icon {
            crate::ui::push_whole(r, lookups, &sheet, row.icon, [1.0; 4]);
        }
        // The name (`:157-167`): White, or Gray with the load-status prefix
        // when invalid (unreachable — the list holds subscribed metadata).
        let (name, tint) = if entry.valid {
            (entry.name.clone(), [1.0; 4])
        } else {
            (format!("(Unknown) {}", entry.name), color::tint([128, 128, 128, 255]))
        };
        text::border_string(r, lookups, row.icon[0] + 60.0 + 6.0, row.icon[1] - 2.0, tint, &name);
        // The workshop badge (`:168`), top-right of the card.
        if let Some(sheet) = lookups.workshop_icon {
            crate::ui::push_whole(r, lookups, &sheet, row.badge, [1.0; 4]);
        }
        // The divider (`:169`): stretched to the card's right edge.
        if let Some(sheet) = lookups.ui_divider {
            crate::ui::push_whole(r, lookups, &sheet, row.divider, [1.0; 4]);
        }
        // The play button (`UIImageButton.DrawSelf`, `:48-56`): White at
        // 1.0 hovered / 0.4 idle (`:13-15`).
        if let Some(sheet) = lookups.card_buttons[0] {
            let vis = if hover && crate::ui::in_rect(mouse, row.play) { 1.0 } else { 0.4 };
            crate::ui::push_whole(r, lookups, &sheet, row.play, [1.0, 1.0, 1.0, vis]);
            // The hover label (`PlayMouseOver`, `:86-89`): "Import" while
            // the button is hovered, "" otherwise.
            if hover && crate::ui::in_rect(mouse, row.play) {
                let label = crate::lang::text("UI.Import");
                text::border_string(r, lookups, row.label_pos[0], row.label_pos[1], [1.0; 4], &label);
            }
        }
        // The three plates (`:170-197`): difficulty, size, created.
        draw_import_plate(r, lookups, row.plate1);
        let (diff_text, diff_color) = crate::ui::world_select::difficulty_text(entry.difficulty);
        draw_import_plate_text(r, lookups, row.plate1, &diff_text, color::tint(diff_color));
        draw_import_plate(r, lookups, row.plate2);
        let size_text = crate::lang::text_args(
            "UI.WorldSizeFormat",
            &[crate::ui::world_select::size_name(entry.size_id)],
        );
        draw_import_plate_text(r, lookups, row.plate2, &size_text, [1.0; 4]);
        draw_import_plate(r, lookups, row.plate3);
        let created = crate::ui::world_select::creation_time(entry.path.as_deref())
            .map(crate::ui::world_select::format_created)
            .unwrap_or_default();
        let created_text = crate::lang::text_args("UI.WorldCreatedFormat", &[created]);
        draw_import_plate_text(r, lookups, row.plate3, &created_text, [1.0; 4]);
    }
    draw_list_scrollbar(r, lookups, &l, state.worlds.len(), 96.0, state.scroll);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource_pack::{PackVersion, ResourcePack};

    fn pack(name: &str, file: &str) -> ResourcePack {
        ResourcePack {
            file_name: file.into(),
            full_path: file.into(),
            name: name.into(),
            description: String::new(),
            author: "Author".into(),
            version: PackVersion { major: 1, minor: 0 },
            is_enabled: false,
            sorting_order: 0,
            is_compressed: false,
        }
    }

    /// `OnInitialize` (`:42-47`): the root is 80% wide, 650 max, Top 220.
    #[test]
    fn the_root_follows_the_oninitialize_dimensions() {
        let l = list_layout(Vec2::new(1600.0, 900.0), 0, 82.0);
        // 0.8*1600 = 1280 -> 650 max; x = (1600-650)/2 = 475.
        assert_eq!(l.root, [475.0, 220.0, 650.0, 680.0]);
        assert_eq!(l.panel, [475.0, 220.0, 650.0, 570.0]);
        // The list fills the panel inner detached.
        assert_eq!(l.list, [487.0, 232.0, 626.0, 546.0]);
        assert!(!l.attached);
    }

    /// `Recalculate` (`:84-102`): the attach branch drops the list width
    /// 25 while the content exceeds the view (7 rows of 87 > 546).
    #[test]
    fn the_scrollbar_attach_branch_flips_the_list_width() {
        let short = list_layout(Vec2::new(1600.0, 900.0), 6, 82.0);
        assert!(!short.attached, "6*87 = 522 <= 546");
        assert_eq!(short.list[2], 626.0);
        let tall = list_layout(Vec2::new(1600.0, 900.0), 7, 82.0);
        assert!(tall.attached, "7*87 = 609 > 546");
        assert_eq!(tall.list[2], 601.0);
        // The import rows attach one row later (96/101).
        assert!(!list_layout(Vec2::new(1600.0, 900.0), 5, 96.0).attached);
        assert!(list_layout(Vec2::new(1600.0, 900.0), 6, 96.0).attached);
    }

    /// `UIList.cs:120-128`: one row drops the padding.
    #[test]
    fn content_heights_drop_the_padding_for_a_single_row() {
        assert_eq!(content_height(0, 82.0), 0.0);
        assert_eq!(content_height(1, 82.0), 82.0);
        assert_eq!(content_height(3, 82.0), 261.0);
        assert_eq!(content_height(1, 96.0), 96.0);
        assert_eq!(content_height(3, 96.0), 303.0);
    }

    /// `PopulateEntries` (`:137-151`): the stable IsCompressed sort keeps
    /// the AllPacks order when nothing is compressed.
    #[test]
    fn populate_keeps_the_allpacks_order_without_zips() {
        let mut s = PublishPacksState::default();
        s.packs.packs = vec![pack("Beta", "b"), pack("Alpha", "a")];
        s.populate();
        assert_eq!(s.rows(), &[1, 0], "Alpha sorts first by name");
        s.packs.packs[1].is_compressed = true;
        s.populate();
        assert_eq!(s.rows(), &[0, 1], "the compressed pack sinks");
    }

    /// The row geometry (`:49-99`, `:193-207`): the 72x72 icon block at
    /// `Left 4, VAlign 0.5`, the plates split the remaining width.
    #[test]
    fn the_pack_row_geometry_follows_the_ctor() {
        let list = [100.0, 200.0, 626.0, 546.0];
        let row = publish_row(list, 1, 72.0, 72.0, [100.0, 40.0], 0.0);
        assert_eq!(row.card, [100.0, 287.0, 626.0, 82.0]); // 87 pitch
        // inner = (106, 293, 614, 70); icon y = 293 + (70-72)/2 = 292.
        assert_eq!(row.icon, [110.0, 292.0, 72.0, 72.0]);
        assert_eq!(row.pack_icon.unwrap(), [113.0, 295.0, 66.0, 66.0]);
        assert_eq!(row.publish, [620.0, 323.0, 100.0, 40.0]);
        assert_eq!(row.name_pos, [190.0, 296.0]);
        // num2 = (614 - 22 - 72 - 100)/2 = 210.
        assert_eq!(row.plate1, [190.0, 323.0, 210.0, 40.0]);
        assert_eq!(row.plate2, [405.0, 323.0, 210.0, 40.0]);
        // The world twin: 60x58 icon, +6 name gap, +40 size plate.
        let w = publish_row(list, 0, 60.0, 58.0, [100.0, 40.0], 40.0);
        // inner y = 206; icon y = 206 + (70-58)/2 = 212.
        assert_eq!(w.icon, [110.0, 212.0, 60.0, 58.0]);
        assert!(w.pack_icon.is_none() && w.icon_border.is_none());
        assert_eq!(w.name_pos, [176.0, 209.0]);
        // num2 = (614 - 22 - 60 - 100)/2 = 216.
        assert_eq!(w.plate2[2], 256.0);
    }

    /// The grey pair (`:109-110`, `:120-121`) — unreachable live (no
    /// zips), pinned for the record.
    #[test]
    fn the_grey_row_colours_match_the_xna_maths() {
        let (bg, border) = publish_row_colors(false, true);
        assert_eq!(bg, [44, 57, 105, 178]);
        assert_eq!(border, [62, 81, 149, 178]);
        let (bg, border) = publish_row_colors(false, false);
        assert_eq!(bg, [49, 56, 80, 178]);
        assert_eq!(border, [88, 88, 88, 178]);
        let (bg, border) = publish_row_colors(true, false);
        assert_eq!(bg, [96, 107, 145, 255]);
        assert_eq!(border, [150, 150, 150, 255]);
    }

    /// The click table: Back, the button's single click, the body's
    /// double-click; a plain body press is dead.
    #[test]
    fn the_click_table_routes_back_button_and_double_click() {
        crate::lang::init(tr_core::localization::GameCulture::English);
        let lookups = assets::Lookups::default();
        let l = list_layout(Vec2::new(1600.0, 900.0), 2, 82.0);
        let action = |rr: [f32; 4], i: usize| pack_action_rect(&lookups, rr, i);
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        assert_eq!(
            click_at(&l, 2, 82.0, 0.0, action, centre(l.back), false, false),
            Click::Back
        );
        let btn = pack_action_rect(&lookups, l.list, 0);
        assert_eq!(
            click_at(&l, 2, 82.0, 0.0, action, centre(btn), false, false),
            Click::Publish(0)
        );
        // The card body away from the button: dead single, live double.
        let body = Vec2::new(l.list[0] + 200.0, l.list[1] + 10.0);
        assert_eq!(click_at(&l, 2, 82.0, 0.0, action, body, false, false), Click::None);
        assert_eq!(click_at(&l, 2, 82.0, 0.0, action, body, true, false), Click::Publish(0));
        // The import flavour names the same hits Import.
        assert_eq!(click_at(&l, 2, 82.0, 0.0, action, body, true, true), Click::Import(0));
        // A scrolled row: the second row's body rides at +87.
        let body2 = Vec2::new(l.list[0] + 200.0, l.list[1] + 97.0);
        assert_eq!(click_at(&l, 2, 82.0, 0.0, action, body2, true, false), Click::Publish(1));
    }

    /// `PopulateEntries` (`UIWorkshopSelectWorldToPublish.cs:134-149`):
    /// favourites first, then name, then file name — no Journey key.
    #[test]
    fn the_publish_world_sort_is_favorite_name_file() {
        let w = |name: &str, fav: bool, file: &str| save_list::WorldEntry {
            name: name.into(),
            difficulty: 0,
            size_id: 1,
            is_favorite: fav,
            valid: true,
            has_been_played: false,
            path: Some(std::path::PathBuf::from(file)),
        };
        let mut worlds = vec![
            w("Beta", false, "b.trwld"),
            w("Alpha", false, "a2.trwld"),
            w("Alpha", false, "a1.trwld"),
            w("Zeta", true, "z.trwld"),
        ];
        sort_publish_worlds(&mut worlds);
        let names: Vec<&str> = worlds.iter().map(|w| w.name.as_str()).collect();
        assert_eq!(names, ["Zeta", "Alpha", "Alpha", "Beta"]);
        assert_eq!(worlds[1].path.as_deref().unwrap().to_str().unwrap(), "a1.trwld");
    }

    /// The import row geometry (`UIWorkshopImportWorldListItem.cs:30-71`,
    /// `:168-196`): the 96 px card at the 101 pitch, the icon at the inner
    /// top, the play button bottom-left, the 100/150/rest plates.
    #[test]
    fn the_import_row_geometry_follows_the_ctor() {
        let list = [100.0, 200.0, 626.0, 546.0];
        let row = import_row(list, 1, [22.0, 22.0], [22.0, 22.0], 4.0, 27.0, 40.0);
        assert_eq!(row.card, [100.0, 301.0, 626.0, 96.0]); // 101 pitch
        // inner = (106, 307, 614, 84); the icon rides the inner TOP.
        assert_eq!(row.icon, [110.0, 307.0, 60.0, 58.0]);
        // play: VAlign 1 of the inner, Left 4.
        assert_eq!(row.play, [110.0, 369.0, 22.0, 22.0]);
        // label: Left 28, VAlign 1 of 16, Top -3, minus the 2 px UIText shift.
        assert_eq!(row.label_pos, [134.0, 370.0]);
        // badge: card.right - 22 - 3, card.y + 2.
        assert_eq!(row.badge, [701.0, 303.0, 22.0, 22.0]);
        // divider: icon right, inner.Y + 21, to the card's right edge.
        assert_eq!(row.divider, [170.0, 328.0, 556.0, 4.0]);
        // plates: y = inner.Y + 29; 100 / 150+40 / the rest.
        assert_eq!(row.plate1, [176.0, 336.0, 100.0, 27.0]);
        assert_eq!(row.plate2, [281.0, 336.0, 190.0, 27.0]);
        // plate3: 281 + 190 + 5 = 476; inner right 720 - 476 = 244.
        assert_eq!(row.plate3, [476.0, 336.0, 244.0, 27.0]);
    }

    /// The import click table (`:39/:45/:46`): the play button's single
    /// click and the body's double-click import; the 96 px rows ride the
    /// 101 pitch.
    #[test]
    fn the_import_click_table_routes_the_play_button() {
        crate::lang::init(tr_core::localization::GameCulture::English);
        let lookups = assets::Lookups::default();
        let l = list_layout(Vec2::new(1600.0, 900.0), 2, 96.0);
        let action = |rr: [f32; 4], i: usize| import_action_rect(&lookups, rr, i);
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        let play = import_action_rect(&lookups, l.list, 0);
        assert_eq!(
            click_at(&l, 2, 96.0, 0.0, action, centre(play), false, true),
            Click::Import(0)
        );
        let body = Vec2::new(l.list[0] + 300.0, l.list[1] + 10.0);
        assert_eq!(click_at(&l, 2, 96.0, 0.0, action, body, false, true), Click::None);
        assert_eq!(click_at(&l, 2, 96.0, 0.0, action, body, true, true), Click::Import(0));
        // The second row rides at +101.
        let body2 = Vec2::new(l.list[0] + 300.0, l.list[1] + 111.0);
        assert_eq!(click_at(&l, 2, 96.0, 0.0, action, body2, true, true), Click::Import(1));
    }
}
