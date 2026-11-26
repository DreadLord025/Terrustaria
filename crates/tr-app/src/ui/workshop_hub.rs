//! `UIWorkshopHub` (`GameContent/UI/States/UIWorkshopHub.cs:13-374`) — the
//! Workshop hub: the 2x2 fancy-button grid (Import Worlds / Use Resource
//! Packs / Publish Worlds / Publish Resource Packs), the hover description
//! panel, Back + Logs. W4-d84 row 1; `FrontScreen::WorkshopHub`.
//!
//! Deviations (docs/ANALYSIS.md "The Workshop/ResourcePack screens"):
//! - The menuMode 11 row only opens the hub when `SocialAPI.Workshop !=
//!   null` (`Main.cs:48937-48947`); the port is no-Steam, so the row stays
//!   the ResourcePacks else branch (`:48949-48957`, already landed) and the
//!   hub is reachable through `TR_UI=wshub` (the screenshot entry).
//! - `GoLogsClick` (`:295-300`) opens `UIReportsPage` — d85 row 10, out of
//!   this range; the click plays 10 and sets the `front_hint` notice.
//! - The gamepad snap points (`SetupGamepadPoints`, `:347-366`) are out of
//!   scope (§J.3).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::ui::text::{self, font_width};
use crate::ui::uistate::{self, Style};
use crate::ui::{color, framework, panel, workshop_kit as kit};

/// The hub's own state — `_previousUIState` (`:15`), the Back target. The
/// menu row constructs it with null (`Main.cs:48944`); `None` runs
/// `menuMode = 0` (`HandleBackButtonUsage`, `:331-334`).
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct HubState {
    /// `_previousUIState` — `None` is the null target (Back closes to the
    /// main menu); `Some` parks the exact screen.
    pub back: Option<Box<crate::app::flow::FrontScreen>>,
}

/// The four grid buttons in the C# append order (`OnInitialize`,
/// `:96-111`): import (top-left), use (top-right), publish-worlds
/// (bottom-left), publish-packs (bottom-right).
pub(crate) const BTN_IMPORT: usize = 0;
pub(crate) const BTN_USE: usize = 1;
pub(crate) const BTN_PUBLISH_WORLDS: usize = 2;
pub(crate) const BTN_PUBLISH_PACKS: usize = 3;

/// The button text keys (`MakeButton_*`, `:122/:136/:150/:164`).
const BUTTON_KEYS: [&str; 4] = [
    "Workshop.HubWorlds",
    "Workshop.HubResourcePacks",
    "Workshop.HubPublishWorlds",
    "Workshop.HubPublishResourcePacks",
];

/// The hover description keys (`ShowOptionDescription`, `:186-209`) —
/// indexed by the button constants.
const DESCRIPTION_KEYS: [&str; 4] = [
    "Workshop.HubDescriptionImportWorlds",
    "Workshop.HubDescriptionUseResourcePacks",
    "Workshop.HubDescriptionPublishWorlds",
    "Workshop.HubDescriptionPublishResourcePacks",
];

/// Every static rect of the hub (`OnInitialize`, `:45-118`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Layout {
    /// The 600-wide root at Top 250, Height 350 (`:48-57`).
    pub root: [f32; 4],
    /// The container panel, Height 260 (`:59-62`).
    pub panel: [f32; 4],
    /// The 154 px button block (`:63-66`, `SetPadding(0)`).
    pub block: [f32; 4],
    /// The four `MakeFancyButton` panels — `Width/Height (-3, 0.5)` of the
    /// block (`:252-253`), the HAlign/VAlign corners (`:97-110`).
    pub buttons: [[f32; 4]; 4],
    /// The horizontal separator (`AddHorizontalSeparator`, `:112/:176-184`):
    /// `Top 166-8 = 158`, `Width (0, 1)`, the sheet's own height.
    pub separator: [f32; 4],
    /// The description highlight (`AddDescriptionPanel`, `:113/:216-230`):
    /// `VAlign 1`, `Top 2`, Height 74, the CategoryPanelHighlight 10-slice.
    pub desc_panel: [f32; 4],
    /// The title panel (`:67-71`): 0.8 large, `Top -46`, padding 15.
    pub title: [f32; 4],
    /// The Back panel (`:72-77`): `Width (-10, 0.5)`, Height 50,
    /// `VAlign 1`, `HAlign 0`, `Top -20` of the ROOT.
    pub back: [f32; 4],
    /// The Logs panel (`:84-89`): the Back twin, `HAlign 1`.
    pub logs: [f32; 4],
}

/// The hub's layout. `title_w` is the measured `UI.WorkshopHub` Death-font
/// width; `sep_h` the Separator1 sheet height (2 fallback).
pub(crate) fn layout(screen: Vec2, title_w: f32, sep_h: f32) -> Layout {
    // `:48-57` — Width 600, Top 250, Height 600-250 = 350, HAlign 0.5.
    let root = uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(250.0),
        Style::px(600.0),
        Style::px(350.0),
        0.5,
        0.0,
        0.0,
        0.0,
        600.0,
        [0.0; 4],
    );
    // `:59-62` — Width (0, 1), Height 600-250-90 = 260.
    let panel = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(0.0, 1.0),
        Style::px(260.0),
        0.0,
        0.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    );
    let panel_inner = uistate::inner(panel, [12.0; 4]);
    // `:63-66` — the button block: Width (0, 1), Height 154, padding 0.
    let block = [panel_inner[0], panel_inner[1], panel_inner[2], 154.0];
    // `MakeFancyButton` (`:252-253`): Width/Height (-3, 0.5); the corners
    // (`:97-110`): import (0,0), use (1,0), publishWorlds (0,1),
    // publishPacks (1,1).
    let button = |h: f32, v: f32| {
        uistate::resolve(
            block,
            Style::px(0.0),
            Style::px(0.0),
            Style::new(-3.0, 0.5),
            Style::new(-3.0, 0.5),
            h,
            v,
            0.0,
            0.0,
            block[2],
            [0.0; 4],
        )
    };
    let buttons = [button(0.0, 0.0), button(1.0, 0.0), button(0.0, 1.0), button(1.0, 1.0)];
    // `AddHorizontalSeparator` (`:176-184`): Top 166-8 = 158 of the panel
    // inner, Width (0, 1), the sheet's height.
    let separator = [panel_inner[0], panel_inner[1] + 158.0, panel_inner[2], sep_h];
    // `AddDescriptionPanel` (`:216-230`): HAlign 0.5, VAlign 1, Width (0, 1),
    // Height 260-154-12-12-8 = 74, Top 2.
    let desc_panel = uistate::resolve(
        panel_inner,
        Style::px(0.0),
        Style::px(2.0),
        Style::new(0.0, 1.0),
        Style::px(74.0),
        0.5,
        1.0,
        0.0,
        0.0,
        panel_inner[2],
        [0.0; 4],
    );
    // `:67-71` — the title: 0.8 large, HAlign 0.5, Top -46, padding 15.
    let title = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-46.0),
        Style::px(0.0),
        Style::px(0.0),
        0.5,
        0.0,
        title_w * 0.8 + 30.0,
        32.0 * 0.8 + 30.0,
        root[2],
        [0.0; 4],
    );
    // `:72-77` / `:84-89` — Back / Logs: Width (-10, 0.5), Height 50,
    // VAlign 1, Top -20, HAlign 0 / 1 (of the ROOT).
    let fat = |h: f32| {
        uistate::resolve(
            root,
            Style::px(0.0),
            Style::px(-20.0),
            Style::new(-10.0, 0.5),
            Style::px(50.0),
            h,
            1.0,
            0.0,
            0.0,
            root[2],
            [0.0; 4],
        )
    };
    Layout {
        root,
        panel,
        block,
        buttons,
        separator,
        desc_panel,
        title,
        back: fat(0.0),
        logs: fat(1.0),
    }
}

/// The element ids: Back, Logs, then the four buttons in append order.
pub(crate) const BACK: u32 = 1;
pub(crate) const LOGS: u32 = 2;
pub(crate) const BUTTON_BASE: u32 = 3;

/// What a click on the hub hit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Click {
    None,
    /// `GoBackClick` (`:289-293`) → `HandleBackButtonUsage` (`:329-339`).
    Back,
    /// `GoLogsClick` (`:295-300`) — the out-of-range reports page.
    Logs,
    /// One of the four grid buttons (`Click_Open*`, `:128-174`).
    Button(usize),
}

/// The flat element list in the C# append order — Back and Logs ride the
/// root before the panel (`:82/:94` vs `:115`), so the reverse walk tests
/// the grid buttons first.
fn elements(l: &Layout) -> Vec<framework::Element> {
    let mut out = vec![
        framework::Element::hit(l.back, BACK),
        framework::Element::hit(l.logs, LOGS),
    ];
    for (i, rect) in l.buttons.iter().enumerate() {
        out.push(framework::Element::hit(*rect, BUTTON_BASE + i as u32));
    }
    out
}

/// The element under the pointer — `App::front_hit`'s feed (the hover-tick
/// edge; the hub has no double-click subscriptions).
pub(crate) fn hit_element(l: &Layout, mouse: Vec2) -> Option<u32> {
    framework::get_element_at(&elements(l), mouse)
}

/// The hub's click table.
pub(crate) fn click_at(l: &Layout, mouse: Vec2) -> Click {
    match hit_element(l, mouse) {
        Some(BACK) => Click::Back,
        Some(LOGS) => Click::Logs,
        Some(id) if id >= BUTTON_BASE => Click::Button((id - BUTTON_BASE) as usize),
        _ => Click::None,
    }
}

/// The hovered grid button (`ShowOptionDescription`, `:186-209`) — `None`
/// shows `Workshop.HubDescriptionDefault` (`ClearOptionDescription`,
/// `:211-214`).
pub(crate) fn hovered_button(l: &Layout, mouse: Vec2) -> Option<usize> {
    match hit_element(l, mouse) {
        Some(id) if id >= BUTTON_BASE => Some((id - BUTTON_BASE) as usize),
        _ => None,
    }
}

/// The description text for the hover state.
pub(crate) fn description_text(hovered: Option<usize>) -> String {
    match hovered {
        Some(i) => crate::lang::text(DESCRIPTION_KEYS[i]),
        None => crate::lang::text("Workshop.HubDescriptionDefault"),
    }
}

/// `UIWorkshopHub.Draw` (`:341-345` → `base.Draw`). `mouse` drives the
/// button hover colours, the description swap and the fat panels' fade.
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    mouse: Vec2,
) {
    let title_text = crate::lang::text("UI.WorkshopHub");
    let title_w = match &lookups.death_font {
        Some(f) => font_width(f, &title_text, 1.0),
        None => r.text_width(&title_text, 1.0),
    };
    let sep_h = lookups.ui_separator.map_or(2.0, |s| s.height as f32);
    let l = layout(screen, title_w, sep_h);

    // The container panel (`:59-62`), bg (33,43,79)*0.8.
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(kit::CLUSTER_PANEL_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.panel, color::tint(uistate::TEXT_PANEL_BORDER));

    // The separator (`:176-184`): Lerp(White,(63,65,151),.85)*.9 — the
    // creation screens' colour; `Utils.DrawPanel` edge 2 (`Utils.cs:
    // 2612-2617`): the 2 px end columns + the stretched middle.
    if let Some(sheet) = lookups.ui_separator {
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let sw = sheet.width as f32;
        let sh = sheet.height as f32;
        let tint = color::tint(uistate::CREATION_SEPARATOR_COLOR);
        let mut piece = |dx: f32, dw: f32, sx: f32, sw2: f32| {
            if dw <= 0.0 {
                return;
            }
            let uv = [
                (sheet.x as f32 + sx) / aw,
                sheet.y as f32 / ah,
                (sheet.x as f32 + sx + sw2) / aw,
                (sheet.y as f32 + sh) / ah,
            ];
            r.push_ui_sprite([l.separator[0] + dx, l.separator[1]], [dw, sh], uv, tint);
        };
        piece(0.0, 2.0, 0.0, 2.0);
        piece(2.0, l.separator[2] - 4.0, 2.0, sw - 4.0);
        piece(l.separator[2] - 2.0, 2.0, sw - 2.0, 2.0);
    }

    // The description highlight (`:219-229`): the CategoryPanelHighlight
    // 10-slice at LightGray*0.7 = (147,147,147,178).
    uistate::draw_spliced_panel(
        r, lookups, lookups.cc_panel_highlight, l.desc_panel,
        10.0, 10.0, 10.0, 10.0,
        color::tint(color::mul([211, 211, 211, 255], 0.7)),
    );
    // The wrapped description (`:231-243`): UIText scale 1 small, Top 5,
    // padding left/right 20, top 6; the element grows to the wrapped
    // MinHeight (`UIText.cs:189`) but stays VAlign 0.
    let hovered = hovered_button(&l, mouse);
    let desc = description_text(hovered);
    let desc_inner_w = l.desc_panel[2] - 40.0;
    let lines = kit::wrap_text(r, lookups, &desc, 1.0, desc_inner_w, false);
    let desc_h = kit::wrapped_height(lookups, lines.len(), 1.0, false, 20.0);
    let _ = desc_h; // VAlign 0 — the element's MinHeight grows, the top stays put
    let line_h = text::line_height(lookups);
    let mut ty = l.desc_panel[1] + 5.0 + 6.0 - 2.0; // Top 5 + PaddingTop 6 - 2*scale
    for line in &lines {
        text::border_string(r, lookups, l.desc_panel[0] + 20.0, ty, [1.0; 4], line);
        ty += line_h;
    }

    // The four fancy buttons (`MakeFancyButton`, `:247-287`).
    for (i, rect) in l.buttons.iter().enumerate() {
        let hover = crate::ui::in_rect(mouse, *rect);
        // `SetColorsToHovered`/`NotHovered` (`:315-327`).
        let (bg, border) = if hover {
            ([73, 94, 171, 255], [89, 116, 213, 255])
        } else {
            (
                color::mul([63, 82, 151, 255], 0.7),
                color::mul([89, 116, 213, 255], 0.7),
            )
        };
        panel::draw_panel(r, lookups, lookups.ui_panel_bg, *rect, color::tint(bg));
        panel::draw_panel(r, lookups, lookups.ui_panel_border, *rect, color::tint(border));
        let inner = uistate::inner(*rect, [6.0; 4]);
        // The icon (`:259-265`): UIImage VAlign 0.5, Left 2, natural size.
        if let Some(icon) = lookups.workshop_hub_icons[i] {
            let ix = inner[0] + 2.0;
            let iy = inner[1] + (inner[3] - icon.height as f32) * 0.5;
            crate::ui::push_whole(
                r, lookups, &icon,
                [ix, iy, icon.width as f32, icon.height as f32],
                [1.0; 4],
            );
        }
        // The label (`:268-284`): UIText 0.45 large, Left 80, Top 5,
        // Width (-80, 1), Height (0, 1), VAlign 0.5, TextOrigin 0,
        // PaddingRight 20, PaddingTop 10, IsWrapped.
        let label = crate::lang::text(BUTTON_KEYS[i]);
        let box_w = inner[2] - 80.0;
        let wrap_w = box_w - 20.0;
        let lines = kit::wrap_text(r, lookups, &label, 0.45, wrap_w, true);
        let text_h = kit::wrapped_height(lookups, lines.len(), 0.45, true, 20.0);
        // The element: Height (0,1) of the inner, MinHeight = text_h + 10
        // (`UIText.cs:189`), VAlign 0.5 + Top 5.
        let elem_h = inner[3].max(text_h + 10.0);
        let elem_y = inner[1] + 5.0 + (inner[3] - elem_h) * 0.5;
        // DrawSelf: inner top (PaddingTop 10) minus 10*scale (large).
        let mut ty = elem_y + 10.0 - 10.0 * 0.45;
        let line_h = text::death_line_height(lookups) * 0.45;
        for line in &lines {
            text::draw_border_string(
                r, lookups, text::BorderFont::Death,
                inner[0] + 80.0, ty, 0.45, &[[1.0, 1.0]], [1.0; 4], line,
            );
            ty += line_h;
        }
    }

    // The title (`:67-71`): 0.8 large, padding 15, TITLE_BG.
    uistate::draw_text_panel(
        r, lookups, l.title,
        &uistate::TextPanelStyle {
            text_scale: 0.8,
            large: true,
            padding: 15.0,
            bg: kit::TITLE_BG,
            border: uistate::TEXT_PANEL_BORDER,
        },
        &title_text, title_w, false,
    );

    // Back / Logs (`:72-95`): 0.7 large, FadedMouseOver/Out — the hub's
    // fade-out bg is (63,82,151)*0.8 (`:311`), NOT the 0.7 twin.
    let faded_out = color::mul([63, 82, 151, 255], 0.8);
    for (rect, key) in [(l.back, "UI.Back"), (l.logs, "Workshop.ReportLogsButton")] {
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
                bg: faded_out,
                border: uistate::TEXT_PANEL_BORDER,
            },
            &label, w, crate::ui::in_rect(mouse, rect),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `OnInitialize` (`:48-66`): the 600-wide root at Top 250, the 260 px
    /// panel, the 154 px block, the (-3, 0.5) corner buttons.
    #[test]
    fn the_layout_follows_the_oninitialize_dimensions() {
        let l = layout(Vec2::new(1600.0, 900.0), 200.0, 2.0);
        assert_eq!(l.root, [500.0, 250.0, 600.0, 350.0]);
        assert_eq!(l.panel, [500.0, 250.0, 600.0, 260.0]);
        // inner = (512, 262, 576, 236); block height 154.
        assert_eq!(l.block, [512.0, 262.0, 576.0, 154.0]);
        // buttons: 576*0.5-3 = 285 wide, 154*0.5-3 = 74 tall.
        assert_eq!(l.buttons[BTN_IMPORT], [512.0, 262.0, 285.0, 74.0]);
        assert_eq!(l.buttons[BTN_USE], [803.0, 262.0, 285.0, 74.0]);
        assert_eq!(l.buttons[BTN_PUBLISH_WORLDS], [512.0, 342.0, 285.0, 74.0]);
        assert_eq!(l.buttons[BTN_PUBLISH_PACKS], [803.0, 342.0, 285.0, 74.0]);
        // separator: inner.Y + 158; description: VAlign 1 + Top 2, h 74.
        assert_eq!(l.separator, [512.0, 420.0, 576.0, 2.0]);
        assert_eq!(l.desc_panel, [512.0, 426.0, 576.0, 74.0]);
        // Back/Logs: root.VAlign 1 - 50 - 20; half width - 10.
        assert_eq!(l.back, [500.0, 530.0, 290.0, 50.0]);
        assert_eq!(l.logs, [810.0, 530.0, 290.0, 50.0]);
    }

    /// The click table: Back, Logs and the four buttons; the description
    /// panel is not a click target (`UISlicedImage` has no handlers).
    #[test]
    fn the_click_table_routes_the_grid_and_fat_buttons() {
        let l = layout(Vec2::new(1600.0, 900.0), 200.0, 2.0);
        let centre = |r: [f32; 4]| Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5);
        assert_eq!(click_at(&l, centre(l.back)), Click::Back);
        assert_eq!(click_at(&l, centre(l.logs)), Click::Logs);
        for i in 0..4 {
            assert_eq!(click_at(&l, centre(l.buttons[i])), Click::Button(i));
            assert_eq!(hovered_button(&l, centre(l.buttons[i])), Some(i));
        }
        assert_eq!(click_at(&l, centre(l.desc_panel)), Click::None);
        assert_eq!(hovered_button(&l, centre(l.desc_panel)), None);
    }

    /// `ShowOptionDescription`/`ClearOptionDescription` (`:186-214`): the
    /// hover swaps the description key, the default restores.
    #[test]
    fn the_description_follows_the_hovered_button() {
        crate::lang::init(tr_core::localization::GameCulture::English);
        assert_eq!(description_text(None), "Choose an option for more information.");
        assert!(description_text(Some(BTN_IMPORT)).starts_with("Import Worlds"));
        assert!(description_text(Some(BTN_USE)).starts_with("Use Resource Packs"));
        assert!(description_text(Some(BTN_PUBLISH_WORLDS)).starts_with("Publish your Worlds"));
        assert!(description_text(Some(BTN_PUBLISH_PACKS)).starts_with("Publish your Resource Packs"));
    }
}
