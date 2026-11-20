//! `UIResourcePackSelectionMenu` (`GameContent/UI/States/
//! UIResourcePackSelectionMenu.cs:17-591`) — the two-list resource-pack
//! screen: available packs on the left, enabled packs (with the priority
//! arrows) on the right, Back + Open Folder below. W4-d84 row 5;
//! `FrontScreen::ResourcePackSelection`.
//!
//! The pack store rides the payload (`_packsList`, `:25`) — the C# mutates
//! the shared `ResourcePack` objects in place; the port's rows carry store
//! indices into it. `PopulatePackList` (`:60-121`) re-derives the two views
//! after every toggle/offset, exactly as the C# rebuilds the element lists.
//!
//! Deviations (docs/ANALYSIS.md "The Workshop/ResourcePack screens"):
//! - `ApplyListChanges` (`:272-275`) feeds `AssetSourceController.
//!   UseResourcePacks` — the port has no content-source pipeline, so the
//!   Back tail stores the enabled-only list in `Settings.resource_packs`
//!   (`Main.SaveSettings`, `:480`) and saves; the asset reload is the
//!   documented no-op.
//! - `Main.instance.ResetAllContentBasedRenderTargets()` (`:162/:204`) has
//!   no port seam (no content render targets) — dropped with the same note.
//! - The hover `MouseText` (`DisplayMouseTextIfHovered`, `:528-535`) rides
//!   the port's `front_hint` notice channel (the world-select rejection's
//!   convention), not a cursor-following tooltip.
//! - The gamepad snap points (`:543-590`) are out of scope (§J.3).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::resource_pack::{ResourcePack, ResourcePackList};
use crate::ui::text::{self, font_width};
use crate::ui::uistate::{self, Style};
use crate::ui::{color, panel, workshop_kit as kit};

/// The screen's own state — `_packsList` (`:25`), the two `UIList` scroll
/// offsets (`_availablePacksList`/`_enabledPacksList`, `:21-23`) and the
/// `_uiStateToGoBackTo` target (`:31`). `None` back = the menuMode 11 entry
/// (`Main.OpenResourcePacksMenu(null)`, `Main.cs:48955`); `Some` = the hub
/// (`Click_OpenResourcePacksMenu`, `UIWorkshopHub.cs:128-132`).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PackSelectionState {
    /// `_packsList` — the flat store the views index into.
    pub packs: ResourcePackList,
    /// The available list's `UIScrollbar.ViewPosition`.
    pub available_scroll: f32,
    /// The enabled list's `UIScrollbar.ViewPosition`.
    pub enabled_scroll: f32,
    /// `_uiStateToGoBackTo` (`:31`) — `None` is the null target (Back runs
    /// `menuMode = 0`, `:486`); `Some` parks the exact screen (the hub).
    pub back: Option<Box<crate::app::flow::FrontScreen>>,
}

impl PackSelectionState {
    /// The ctor (`:51-58`): `BuildPage` then `_packsList = current` +
    /// `PopulatePackList` — the port's `new` takes the discovered list
    /// (`AssetInitializer.CreateResourcePackList`, `Main.cs:52374`) and
    /// runs the priority cleanup the populate does (`:64`).
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// `PopulatePackList` (`:60-121`): the cleanup (`:64`) then the two
    /// sorted views. The port re-derives the views on demand (the C#
    /// rebuilds the element lists; the indices are the same order).
    pub(crate) fn populate(&mut self) {
        self.packs.clean_up_priority();
    }

    /// The available (disabled) view — `DisabledPacks` (`:66`), store indices.
    pub(crate) fn available(&self) -> Vec<usize> {
        self.packs.disabled_packs()
    }

    /// The enabled view — `EnabledPacks` (`:65`), store indices.
    pub(crate) fn enabled(&self) -> Vec<usize> {
        self.packs.enabled_packs()
    }
}

/// Every static rect of the screen (`BuildPage`, `:287-420`). The two lists
/// keep their own scroll offsets; `available_w`/`enabled_w` carry the
/// `(-25, 1)` widths (the scrollbars are always children here — the C#
/// appends them unconditionally, `:379-386`/`:412-418`, no `Recalculate`
/// detach branch on this screen).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Layout {
    /// The 80%-wide, 800-max/600-min root (`:290-297`).
    pub root: [f32; 4],
    /// The container panel, `Height (-110, 1)`, `PaddingLeft/Right 0`
    /// (`:298-306`).
    pub panel: [f32; 4],
    /// The left column element — `Width (-20, 0.5)`, `Left 10` (`:318-325`).
    pub left_col: [f32; 4],
    /// The right column element — `Width (-20, 0.5)`, `Left -10`,
    /// `HAlign 1` (`:326-334`).
    pub right_col: [f32; 4],
    /// `_availablePacksList` — `Width (-25, 1)`, `HAlign 1` of the left
    /// column (`:335-343`).
    pub available_list: [f32; 4],
    /// `_enabledPacksList` — `Width (-25, 1)`, `HAlign 0` of the right
    /// column (`:344-353`).
    pub enabled_list: [f32; 4],
    /// The available list's scrollbar — `HAlign 0` of the left column
    /// (`:379-386`).
    pub available_bar: [f32; 4],
    /// The enabled list's scrollbar — `HAlign 1` of the right column
    /// (`:412-418`).
    pub enabled_bar: [f32; 4],
    /// The two column titles (`:354-369`): `Top 10`, `Left ±25`.
    pub available_title: [f32; 2],
    pub enabled_title: [f32; 2],
    /// The `UI.ResourcePacks` header panel (`:370-378`): 1.0 large,
    /// `Top -44`, `SetPadding(13)`, bg `TITLE_BG`.
    pub header: [f32; 4],
    /// The vertical separator (`:387-393`): `HAlign 0.5`, `VAlign 1`,
    /// `Height (-12, 1)`, 1 px wide (`UIVerticalSeparator.cs:20`).
    pub separator: [f32; 4],
    /// The Back panel (`:436-448`): `Width (-8, 0.5)`, `Height 50`,
    /// `VAlign 1`, `HAlign 0`, `Top -45`.
    pub back: [f32; 4],
    /// The Open Folder panel (`:449-461`): the Back twin, `HAlign 1`.
    pub open_folder: [f32; 4],
}

/// The screen's layout (`BuildPage`, `:287-420`). `header_w` is the measured
/// `UI.ResourcePacks` Death-font width.
pub(crate) fn layout(screen: Vec2, header_w: f32) -> Layout {
    // `:290-297` — Width (0, 0.8), MaxWidth 800, MinWidth 600, Top 240,
    // Height (-240, 1), HAlign 0.5.
    let root = uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(240.0),
        Style::new(0.0, 0.8),
        Style::new(-240.0, 1.0),
        0.5,
        0.0,
        600.0,
        0.0,
        800.0,
        [0.0; 4],
    );
    // `:298-306` — Width Fill, Height (-110, 1), PaddingLeft/Right 0 (the
    // top/bottom keep UIPanel's 12).
    let panel = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(0.0, 1.0),
        Style::new(-110.0, 1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    );
    let panel_inner = uistate::inner(panel, [0.0, 12.0, 0.0, 12.0]);
    // `:307-317` — the bottom block: Height (-(30+4+5), 1) = -39, VAlign 1,
    // padding 0.
    let block = uistate::resolve(
        panel_inner,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(0.0, 1.0),
        Style::new(-39.0, 1.0),
        0.0,
        1.0,
        0.0,
        0.0,
        panel_inner[2],
        [0.0; 4],
    );
    // `:318-325` — the left column: Width (-20, 0.5), Height (0, 1), Left 10.
    let left_col = uistate::resolve(
        block,
        Style::px(10.0),
        Style::px(0.0),
        Style::new(-20.0, 0.5),
        Style::new(0.0, 1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        block[2],
        [0.0; 4],
    );
    // `:326-334` — the right column: Width (-20, 0.5), Left -10, HAlign 1.
    let right_col = uistate::resolve(
        block,
        Style::px(-10.0),
        Style::px(0.0),
        Style::new(-20.0, 0.5),
        Style::new(0.0, 1.0),
        1.0,
        0.0,
        0.0,
        0.0,
        block[2],
        [0.0; 4],
    );
    // `:335-343` — the available list: Width (-25, 1), Height (0, 1),
    // HAlign 1 of the left column.
    let available_list = uistate::resolve(
        left_col,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(-25.0, 1.0),
        Style::new(0.0, 1.0),
        1.0,
        0.0,
        0.0,
        0.0,
        left_col[2],
        [0.0; 4],
    );
    // `:344-353` — the enabled list: Width (-25, 1), HAlign 0, Left 0.
    let enabled_list = uistate::resolve(
        right_col,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(-25.0, 1.0),
        Style::new(0.0, 1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        right_col[2],
        [0.0; 4],
    );
    // `:379-386` — the available scrollbar: Height (0, 1), HAlign 0, Left 0.
    let available_bar = uistate::resolve(
        left_col,
        Style::px(0.0),
        Style::px(0.0),
        Style::px(20.0),
        Style::new(0.0, 1.0),
        0.0,
        0.0,
        0.0,
        0.0,
        20.0,
        [0.0; 4],
    );
    // `:412-418` — the enabled scrollbar: Height (0, 1), HAlign 1.
    let enabled_bar = uistate::resolve(
        right_col,
        Style::px(0.0),
        Style::px(0.0),
        Style::px(20.0),
        Style::new(0.0, 1.0),
        1.0,
        0.0,
        0.0,
        0.0,
        20.0,
        [0.0; 4],
    );
    // `:354-369` — the titles: Top 10, Left 25 / -25 (HAlign 0 / 1).
    let available_title = [panel_inner[0] + 25.0, panel_inner[1] + 10.0];
    let enabled_title = [panel_inner[0] + panel_inner[2] - 25.0, panel_inner[1] + 10.0];
    // `:370-378` — the header: 1.0 large, HAlign 0.5, Top -44, padding 13.
    let header = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-44.0),
        Style::px(0.0),
        Style::px(0.0),
        0.5,
        0.0,
        header_w + 26.0,
        32.0 + 26.0,
        root[2],
        [0.0; 4],
    );
    // `:387-393` — the vertical separator: HAlign 0.5, VAlign 1,
    // Height (-12, 1), 1 px wide (OnePixel).
    let separator = uistate::resolve(
        panel_inner,
        Style::px(0.0),
        Style::px(0.0),
        Style::px(1.0),
        Style::new(-12.0, 1.0),
        0.5,
        1.0,
        0.0,
        0.0,
        1.0,
        [0.0; 4],
    );
    // `:436-448` — Back: Width (-8, 0.5), Height 50, VAlign 1, HAlign 0,
    // Top -45 (of the ROOT).
    let back = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-45.0),
        Style::new(-8.0, 0.5),
        Style::px(50.0),
        0.0,
        1.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    );
    // `:449-461` — Open Folder: the Back twin, HAlign 1.
    let open_folder = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-45.0),
        Style::new(-8.0, 0.5),
        Style::px(50.0),
        1.0,
        1.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    );
    Layout {
        root,
        panel,
        left_col,
        right_col,
        available_list,
        enabled_list,
        available_bar,
        enabled_bar,
        available_title,
        enabled_title,
        header,
        separator,
        back,
        open_folder,
    }
}

/// One row's button rects inside a `UIResourcePack` card's `ContentPanel`.
/// The available row carries the toggle (right half) + info (left half);
/// the enabled row splits into quarters: toggle, info, order-up, order-down
/// (`PopulatePackList`, `:68-119`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct RowButtons {
    /// The enable/disable toggle (`CreatePackToggleButton`, `:176-207`).
    pub toggle: [f32; 4],
    /// The info button (`CreatePackInfoButton`, `:242-261`).
    pub info: [f32; 4],
    /// The order-up arrow (enabled rows only, `CreateOffsetButton(-1)`,
    /// `:106-110`): `Left 2/3`, `Width 1/6`.
    pub order_up: Option<[f32; 4]>,
    /// The order-down arrow (`CreateOffsetButton(1)`, `:111-115`):
    /// `Left 0.8333334`, `Width 1/6`.
    pub order_down: Option<[f32; 4]>,
}

/// The button rects inside a `ContentPanel` rect — the C# `StyleDimension`
/// percents (`PopulatePackList`, `:94-115`). The one copy the click table
/// (via [`row_buttons`]) and the draw share.
pub(crate) fn content_buttons(c: [f32; 4], enabled_row: bool) -> RowButtons {
    if enabled_row {
        // `:94-115` — toggle Left 0 Width 0.5; info Left 0.5 Width 1/6;
        // up Left 2/3 Width 1/6; down Left 0.8333334 Width 1/6.
        RowButtons {
            toggle: [c[0], c[1], c[2] * 0.5, c[3]],
            info: [c[0] + c[2] * 0.5, c[1], c[2] / 6.0, c[3]],
            order_up: Some([c[0] + c[2] * (2.0 / 3.0), c[1], c[2] / 6.0, c[3]]),
            order_down: Some([c[0] + c[2] * 0.8333334, c[1], c[2] / 6.0, c[3]]),
        }
    } else {
        // `:74-81` — the toggle keeps the ctor's Left 0.5 Width 0.5
        // (`:180-181`); the info keeps Width 0.5 (`:245`), Left unset (0).
        RowButtons {
            toggle: [c[0] + c[2] * 0.5, c[1], c[2] * 0.5, c[3]],
            info: [c[0], c[1], c[2] * 0.5, c[3]],
            order_up: None,
            order_down: None,
        }
    }
}

/// The button rects of row `index` in `list_rect` (the scrolled list). The
/// card geometry rides `kit::pack_card`; the content panel's fractions are
/// [`content_buttons`].
pub(crate) fn row_buttons(
    list_rect: [f32; 4],
    index: usize,
    enabled_row: bool,
    divider_h: f32,
) -> RowButtons {
    let card = kit::pack_card(list_rect, index, divider_h);
    content_buttons(card.content, enabled_row)
}

/// What a click on the screen hit — the `Click_*` handlers' targets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Click {
    None,
    /// `GoBackClick` (`:471-474`).
    Back,
    /// `OpenFoldersClick` (`:464-469`).
    OpenFolder,
    /// The toggle on available row `i` (`EnablePackUpdate`'s button, `:75`).
    Enable(usize),
    /// The toggle on enabled row `i` (`DisablePackUpdate`'s button, `:97`).
    Disable(usize),
    /// The info button on row `i` of either list (`Click_Info`, `:263-270`).
    Info(usize),
    /// The order-up arrow on enabled row `i` (`OffsetBackwardUpdate`, `:171`).
    OrderUp(usize),
    /// The order-down arrow on enabled row `i` (`OffsetFrontwardUpdate`, `:167`).
    OrderDown(usize),
}

/// The screen's click table over the two scrolled lists. `available`/
/// `enabled` are the current views (store indices); the row rects ride the
/// 107 px pitch. The scrollbar rects are NOT click targets here — the
/// App's `front_scrollbar_press` takes them first (the `evt.Target == this`
/// gate, `UIScrollbar.cs:126-143`).
pub(crate) fn click_at(
    l: &Layout,
    available: &[usize],
    enabled: &[usize],
    available_scroll: f32,
    enabled_scroll: f32,
    divider_h: f32,
    mouse: Vec2,
) -> Click {
    if crate::ui::in_rect(mouse, l.back) {
        return Click::Back;
    }
    if crate::ui::in_rect(mouse, l.open_folder) {
        return Click::OpenFolder;
    }
    // The available list: the viewport filter (UIList.cs:59's OverflowHidden)
    // then the row's two buttons.
    let avail_rect = [
        l.available_list[0],
        l.available_list[1] - available_scroll,
        l.available_list[2],
        l.available_list[3],
    ];
    if crate::ui::in_rect(mouse, l.available_list) {
        for (row, &store) in available.iter().enumerate() {
            let b = row_buttons(avail_rect, row, false, divider_h);
            if crate::ui::in_rect(mouse, b.toggle) {
                return Click::Enable(store);
            }
            if crate::ui::in_rect(mouse, b.info) {
                return Click::Info(store);
            }
        }
    }
    let en_rect = [
        l.enabled_list[0],
        l.enabled_list[1] - enabled_scroll,
        l.enabled_list[2],
        l.enabled_list[3],
    ];
    if crate::ui::in_rect(mouse, l.enabled_list) {
        for (row, &store) in enabled.iter().enumerate() {
            let b = row_buttons(en_rect, row, true, divider_h);
            if crate::ui::in_rect(mouse, b.toggle) {
                return Click::Disable(store);
            }
            if crate::ui::in_rect(mouse, b.info) {
                return Click::Info(store);
            }
            if b.order_up.is_some_and(|r| crate::ui::in_rect(mouse, r)) {
                return Click::OrderUp(store);
            }
            if b.order_down.is_some_and(|r| crate::ui::in_rect(mouse, r)) {
                return Click::OrderDown(store);
            }
        }
    }
    Click::None
}

/// The hover notice key (`DisplayMouseTextIfHovered`, `:503-535`) — the
/// `OnUpdate` handlers' `MouseText` over the button under the pointer.
pub(crate) fn hover_text_key(
    l: &Layout,
    available: &[usize],
    enabled: &[usize],
    available_scroll: f32,
    enabled_scroll: f32,
    divider_h: f32,
    mouse: Vec2,
) -> Option<&'static str> {
    match click_at(l, available, enabled, available_scroll, enabled_scroll, divider_h, mouse) {
        Click::Enable(_) => Some("UI.EnableTexturePack"),
        Click::Disable(_) => Some("UI.DisableTexturePack"),
        Click::Info(_) => Some("UI.SeeTexturePackInfo"),
        // `OffsetBackwardUpdate`/`OffsetFrontwardUpdate` (`:503-511`): the
        // up arrow shows "Down" (move backwards), the down arrow "Up" —
        // the C# keys are swapped relative to the visual direction.
        Click::OrderUp(_) => Some("UI.OffsetTexturePackPriorityDown"),
        Click::OrderDown(_) => Some("UI.OffsetTexturePackPriorityUp"),
        _ => None,
    }
}

/// The two column titles (`UpdateTitles`, `:422-432`):
/// `UI.AvailableResourcePacksTitle`/`UI.EnabledResourcePacksTitle` with the
/// `{Amount}` substitution.
pub(crate) fn titles(available: usize, enabled: usize) -> (String, String) {
    use tr_core::localization::Substitution;
    (
        crate::lang::text_format_with(
            "UI.AvailableResourcePacksTitle",
            &[("Amount", Substitution::Int(available as i64))],
        ),
        crate::lang::text_format_with(
            "UI.EnabledResourcePacksTitle",
            &[("Amount", Substitution::Int(enabled as i64))],
        ),
    )
}

// --- The draw (UIResourcePackSelectionMenu.Draw -> the element tree) --------

/// `Draw` (`:537-541` → `base.Draw`): the panel, the two lists of
/// `UIResourcePack` cards, the titles, the header, the separator and the two
/// fat buttons. `mouse` drives the hover colours + the card hover.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &PackSelectionState,
    screen: Vec2,
    mouse: Vec2,
) {
    let header_text = crate::lang::text("UI.ResourcePacks");
    let header_w = match &lookups.death_font {
        Some(f) => font_width(f, &header_text, 1.0),
        None => r.text_width(&header_text, 1.0),
    };
    let l = layout(screen, header_w);
    let available = state.available();
    let enabled = state.enabled();
    let divider_h = lookups.ui_divider.map_or(4.0, |s| s.height as f32);

    // The container panel (`:298-306`), bg (33,43,79)*0.8.
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(kit::CLUSTER_PANEL_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.panel, color::tint(uistate::TEXT_PANEL_BORDER));

    // The vertical separator (`:387-393`): (89,116,213)*0.9, 1 px wide —
    // the OnePixel sheet's solid quad (UIVerticalSeparator.cs:19-27), the
    // white-uv convention the achievements/bestiary bars use.
    let sep_color = color::tint(color::mul([89, 116, 213, 255], 0.9));
    let white = r.white_uv();
    r.push_ui_sprite(
        [l.separator[0], l.separator[1]],
        [l.separator[2], l.separator[3]],
        white,
        sep_color,
    );

    // The two lists.
    draw_list(r, lookups, state, &l, &available, false, divider_h, mouse);
    draw_list(r, lookups, state, &l, &enabled, true, divider_h, mouse);

    // The titles (`:354-369`, `UpdateTitles`): UIText scale 1, the enabled
    // one right-aligned (HAlign 1 + TextOriginX 0.5 over its -25-wide box).
    let (avail_title, en_title) = titles(available.len(), enabled.len());
    text::border_string(r, lookups, l.available_title[0], l.available_title[1] - 2.0, [1.0; 4], &avail_title);
    let en_w = text::hud_text_width(r, lookups, &en_title, 1.0);
    text::border_string(r, lookups, l.enabled_title[0] - en_w, l.enabled_title[1] - 2.0, [1.0; 4], &en_title);

    // The header (`:370-378`): 1.0 large, padding 13, TITLE_BG.
    uistate::draw_text_panel(
        r, lookups, l.header,
        &uistate::TextPanelStyle {
            text_scale: 1.0,
            large: true,
            padding: 13.0,
            bg: kit::TITLE_BG,
            border: uistate::TEXT_PANEL_BORDER,
        },
        &header_text, header_w, false,
    );

    // The two fat buttons (`AddBackAndFolderButtons`, `:434-462`): 0.7
    // large, FadedMouseOver/Out.
    for (rect, key) in [(l.back, "UI.Back"), (l.open_folder, "GameUI.OpenFileFolder")] {
        let label = crate::lang::text(key);
        let w = match &lookups.death_font {
            Some(f) => font_width(f, &label, 1.0),
            None => r.text_width(&label, 1.0),
        };
        uistate::draw_text_panel(
            r, lookups, rect,
            &uistate::TextPanelStyle {
                text_scale: 0.7,
                large: true,
                padding: 12.0,
                bg: uistate::TEXT_PANEL_BG,
                border: uistate::TEXT_PANEL_BORDER,
            },
            &label, w, crate::ui::in_rect(mouse, rect),
        );
    }
}

/// One `UIList` of `UIResourcePack` cards (`PopulatePackList`, `:60-121`):
/// the rows at the 107 px pitch, lifted by the list's scroll, clipped to the
/// viewport; each card draws the panel, the icon block, the name/author/
/// divider and the button strip.
#[allow(clippy::too_many_arguments)]
fn draw_list(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &PackSelectionState,
    l: &Layout,
    view: &[usize],
    enabled_list: bool,
    divider_h: f32,
    mouse: Vec2,
) {
    let (list, scroll, bar) = if enabled_list {
        (l.enabled_list, state.enabled_scroll, l.enabled_bar)
    } else {
        (l.available_list, state.available_scroll, l.available_bar)
    };
    let content_h = kit::pack_content_height(view.len());
    let row_rect = [list[0], list[1] - scroll, list[2], list[3]];
    for (row, &store) in view.iter().enumerate() {
        let card = kit::pack_card(row_rect, row, divider_h);
        // The viewport filter (UIList.cs:59's OverflowHidden).
        if card.card[1] + card.card[3] <= list[1] || card.card[1] >= list[1] + list[3] {
            continue;
        }
        draw_pack_card(r, lookups, &state.packs.packs[store], &card, enabled_list, row, view.len(), mouse);
    }
    // The scrollbar (always a child on this screen, `:379-386`/`:412-418`;
    // AutoHide is false — the ctor never sets it, `UIScrollbar.cs:29`).
    panel::draw_scrollbar(r, lookups, bar, list[3], content_h, scroll, false);
}

/// One `UIResourcePack` card (`UIResourcePack.cs:40-114` +
/// `PopulatePackList`'s buttons). `enabled_row` picks the four-button strip;
/// `row`/`count` feed the offset arrows' dead-end gate (`:131`).
#[allow(clippy::too_many_arguments)]
fn draw_pack_card(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    pack: &ResourcePack,
    card: &kit::PackCard,
    enabled_row: bool,
    row: usize,
    count: usize,
    mouse: Vec2,
) {
    let hover = crate::ui::in_rect(mouse, card.card);
    let (bg, border) = if hover {
        (kit::PACK_BG_HOVER, kit::PACK_BORDER_HOVER)
    } else {
        (kit::PACK_BG_REST, kit::PACK_BORDER_REST)
    };
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, card.card, color::tint(bg));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, card.card, color::tint(border));
    // The icon block (`DrawIcon`, `:109-114`): the fallback icon + the
    // Achievement_Borders frame.
    if let Some(icon) = lookups.default_pack_icon {
        crate::ui::push_whole(r, lookups, &icon, card.icon, [1.0; 4]);
    }
    if let Some(borders) = lookups.ach_borders {
        crate::ui::push_whole(r, lookups, &borders, card.icon_border, [1.0; 4]);
    }
    // The name (UIText scale 1, `:61-65`) + the author (0.7, `:68-72`).
    text::border_string(r, lookups, card.name_pos[0], card.name_pos[1] - 2.0, [1.0; 4], &pack.name);
    let author = kit::author_line(pack);
    text::draw_border_string(
        r, lookups, text::BorderFont::Ui, card.author_pos[0], card.author_pos[1] - 2.0 * 0.7,
        0.7, &[[1.0, 1.0]], [1.0; 4], &author,
    );
    // The divider (`:74-83`): ScaleToFit over the (-80, 1) box.
    if let Some(sheet) = lookups.ui_divider {
        crate::ui::push_whole(r, lookups, &sheet, card.divider, [1.0; 4]);
    }
    // The button strip — the content panel's fractions over the card.
    let b = content_buttons(card.content, enabled_row);
    // The toggle (`CreatePackToggleButton`, `:176-207`): LightGreen when
    // enabled, PaleVioletRed when not, both at 0.7 (`:183`); the icon frame
    // is row 1 of TexturePackButtons, col 0 enabled / col 1 disabled (`:188`).
    let toggle_color = if pack.is_enabled {
        color::tint(color::mul([144, 238, 144, 255], 0.7)) // Color.LightGreen
    } else {
        color::tint(color::mul([219, 112, 147, 255], 0.7)) // Color.PaleVioletRed
    };
    draw_group_button(r, lookups, b.toggle, toggle_color, crate::ui::in_rect(mouse, b.toggle));
    draw_button_frame(r, lookups, b.toggle, if pack.is_enabled { 0 } else { 1 }, 1);
    // The info button (`CreatePackInfoButton`, `:242-261`): the
    // UIResourcePackInfoButton's PanelGrayscale at Lerp(Black, _color, 0.8)*0.5
    // (`UIResourcePackInfoButton.cs:43`) + the CharInfo icon centred.
    let info_hover = crate::ui::in_rect(mouse, b.info);
    let info_tint = color::tint(color::mul(
        kit::lerp_bytes([0; 4], uistate::INVENTORY_DEFAULT_COLOR, 0.8), 0.5,
    ));
    uistate::draw_spliced_panel(r, lookups, lookups.cc_panel_gray, b.info, 10.0, 10.0, 10.0, 10.0, info_tint);
    if info_hover {
        uistate::draw_spliced_panel(r, lookups, lookups.cc_panel_border, b.info, 10.0, 10.0, 10.0, 10.0, [1.0; 4]);
    }
    if let Some(info) = lookups.cc_char_info {
        let ix = b.info[0] + (b.info[2] - info.width as f32) * 0.5;
        let iy = b.info[1] + (b.info[3] - info.height as f32) * 0.5;
        crate::ui::push_whole(r, lookups, &info, [ix, iy, info.width as f32, info.height as f32], [1.0; 4]);
    }
    // The offset arrows (enabled rows only, `CreateOffsetButton`, `:123-174`):
    // LightCyan at 0.7 both states (`:132-133`), the TexturePackButtons row-0
    // frames; the dead-end gate (`:131`) greys nothing — the click just plays
    // 12 without moving (`:148-154`), so the draw is identical.
    if let (Some(up), Some(down)) = (b.order_up, b.order_down) {
        let cyan = color::tint(color::mul([224, 255, 255, 255], 0.7)); // Color.LightCyan
        let _ = (row, count); // the gate is click-side (`:131`), not draw-side
        draw_group_button(r, lookups, up, cyan, crate::ui::in_rect(mouse, up));
        draw_button_frame(r, lookups, up, 0, 0);
        draw_group_button(r, lookups, down, cyan, crate::ui::in_rect(mouse, down));
        draw_button_frame(r, lookups, down, 1, 0);
    }
}

/// `GroupOptionButton.DrawSelf`'s base panel (`GroupOptionButton.cs:233`):
/// the PanelGrayscale 10-slice at `Lerp(Black, color, FadeFromBlack=1) *
/// opacity` — the colour already carries the opacity here. The hover border
/// (`:238-241`) is the CategoryPanelBorder at `_borderColor` (the ctor's
/// `Color.White` default, `:28`... the selection menu never calls
/// SetBorderColor, so the hover border stays the ctor default).
fn draw_group_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    tint: [f32; 4],
    hover: bool,
) {
    uistate::draw_spliced_panel(r, lookups, lookups.cc_panel_gray, rect, 10.0, 10.0, 10.0, 10.0, tint);
    if hover {
        uistate::draw_spliced_panel(r, lookups, lookups.cc_panel_border, rect, 10.0, 10.0, 10.0, 10.0, [1.0; 4]);
    }
}

/// The `UIImageFramed` icon centred in the button (`:137-142`/`:188-193`):
/// `asset.Frame(2, 2, col, row)` — the 2x2 TexturePackButtons sheet's quarter.
fn draw_button_frame(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    col: i32,
    row: i32,
) {
    let Some(sheet) = lookups.texture_pack_buttons else { return };
    let fw = sheet.width as f32 / 2.0;
    let fh = sheet.height as f32 / 2.0;
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let x0 = sheet.x as f32 + fw * col as f32;
    let y0 = sheet.y as f32 + fh * row as f32;
    let uv = [x0 / aw, y0 / ah, (x0 + fw) / aw, (y0 + fh) / ah];
    let dx = rect[0] + (rect[2] - fw) * 0.5;
    let dy = rect[1] + (rect[3] - fh) * 0.5;
    r.push_ui_sprite([dx, dy], [fw, fh], uv, [1.0; 4]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resource_pack::PackVersion;

    fn pack(name: &str, file: &str, enabled: bool, order: i32) -> ResourcePack {
        ResourcePack {
            file_name: file.into(),
            full_path: file.into(),
            name: name.into(),
            description: String::new(),
            author: String::new(),
            version: PackVersion { major: 1, minor: 0 },
            is_enabled: enabled,
            sorting_order: order,
            is_compressed: false,
        }
    }

    fn state() -> PackSelectionState {
        let mut s = PackSelectionState::new();
        s.packs.packs = vec![
            pack("Alpha", "a", true, 0),
            pack("Beta", "b", true, 1),
            pack("Gamma", "g", false, 0),
        ];
        s.populate();
        s
    }

    /// `BuildPage` (`:290-297`): the root is 80% wide, 800 max / 600 min,
    /// Top 240, centred.
    #[test]
    fn the_root_follows_the_buildpage_dimensions() {
        let l = layout(Vec2::new(1600.0, 900.0), 200.0);
        // 0.8*1600 = 1280 -> 800 max; x = (1600-800)/2 = 400.
        assert_eq!(l.root, [400.0, 240.0, 800.0, 660.0]);
        let l = layout(Vec2::new(600.0, 900.0), 200.0);
        assert_eq!(l.root[2], 600.0, "the 600 min");
    }

    /// `PopulatePackList` (`:60-121`): the views split by IsEnabled and the
    /// cleanup renumbers the enabled orders.
    #[test]
    fn populate_splits_the_views_and_cleans_the_priority() {
        let s = state();
        assert_eq!(s.available(), vec![2]);
        assert_eq!(s.enabled(), vec![0, 1]);
        // A gap in the orders closes (`CleanUpResourcePackPriority`).
        let mut s = state();
        s.packs.packs[1].sorting_order = 40;
        s.populate();
        assert_eq!(s.packs.packs[0].sorting_order, 0);
        assert_eq!(s.packs.packs[1].sorting_order, 1);
    }

    /// The toggle click flips IsEnabled then runs SetResourcePackAsTopPriority
    /// (`:198-205`): enabling lands above every other enabled pack.
    #[test]
    fn enabling_a_pack_lands_it_at_top_priority() {
        let mut s = state();
        // Click the available row's toggle (Gamma, store 2).
        s.packs.packs[2].is_enabled = !s.packs.packs[2].is_enabled;
        s.packs.set_top_priority(2);
        s.populate();
        assert_eq!(s.enabled(), vec![0, 1, 2], "Gamma lands last (top)");
        assert_eq!(s.packs.packs[2].sorting_order, 2);
    }

    /// The offset arrows swap with the clamped neighbour
    /// (`OffsetResourcePackPriority`, `:226-240`).
    #[test]
    fn the_offset_arrows_swap_the_clamped_neighbour() {
        let mut s = state();
        s.packs.offset_priority(0, 1); // Alpha <-> Beta
        s.populate();
        assert_eq!(s.enabled(), vec![1, 0]);
        s.packs.offset_priority(0, 1); // Beta at the bottom: clamped, no-op
        assert_eq!(s.packs.packs[0].sorting_order, 1);
    }

    /// `row_buttons`' fractions (`:94-115`): the enabled row's quarters, the
    /// available row's halves.
    #[test]
    fn the_button_strip_follows_the_styledimension_percents() {
        let list = [0.0, 0.0, 400.0, 500.0];
        let b = row_buttons(list, 0, true, 4.0);
        let c = kit::pack_card(list, 0, 4.0).content;
        assert_eq!(b.toggle, [c[0], c[1], c[2] * 0.5, c[3]]);
        assert_eq!(b.info, [c[0] + c[2] * 0.5, c[1], c[2] / 6.0, c[3]]);
        assert_eq!(b.order_up.unwrap()[0], c[0] + c[2] * (2.0 / 3.0));
        assert_eq!(b.order_down.unwrap()[0], c[0] + c[2] * 0.8333334);
        let a = row_buttons(list, 0, false, 4.0);
        assert!(a.order_up.is_none() && a.order_down.is_none());
        assert_eq!(a.toggle[0], c[0] + c[2] * 0.5);
        assert_eq!(a.info, [c[0], c[1], c[2] * 0.5, c[3]]);
    }

    /// The click table routes each button to its handler target.
    #[test]
    fn the_click_table_routes_the_buttons() {
        let l = layout(Vec2::new(1600.0, 900.0), 200.0);
        let s = state();
        let (avail, en) = (s.available(), s.enabled());
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        // Back / Open Folder.
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(l.back)), Click::Back);
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(l.open_folder)), Click::OpenFolder);
        // The available row's toggle + info.
        let ab = row_buttons(l.available_list, 0, false, 4.0);
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(ab.toggle)), Click::Enable(2));
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(ab.info)), Click::Info(2));
        // The enabled row's four buttons.
        let eb = row_buttons(l.enabled_list, 0, true, 4.0);
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(eb.toggle)), Click::Disable(0));
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(eb.info)), Click::Info(0));
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(eb.order_up.unwrap())), Click::OrderUp(0));
        assert_eq!(click_at(&l, &avail, &en, 0.0, 0.0, 4.0, centre(eb.order_down.unwrap())), Click::OrderDown(0));
    }

    /// The hover keys (`:503-535`) — the offset arrows' swapped wording.
    #[test]
    fn the_hover_keys_match_the_onupdate_handlers() {
        let l = layout(Vec2::new(1600.0, 900.0), 200.0);
        let s = state();
        let (avail, en) = (s.available(), s.enabled());
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        let eb = row_buttons(l.enabled_list, 0, true, 4.0);
        assert_eq!(
            hover_text_key(&l, &avail, &en, 0.0, 0.0, 4.0, centre(eb.order_up.unwrap())),
            Some("UI.OffsetTexturePackPriorityDown")
        );
        assert_eq!(
            hover_text_key(&l, &avail, &en, 0.0, 0.0, 4.0, centre(eb.order_down.unwrap())),
            Some("UI.OffsetTexturePackPriorityUp")
        );
    }

    /// `UpdateTitles` (`:422-432`): the {Amount} substitution.
    #[test]
    fn the_titles_carry_the_amounts() {
        crate::lang::init(tr_core::localization::GameCulture::English);
        let (a, e) = titles(3, 2);
        assert_eq!(a, "Available Packs (3)");
        assert_eq!(e, "Enabled Packs (2)");
    }
}
