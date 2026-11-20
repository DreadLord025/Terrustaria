//! `UIResourcePackInfoMenu` (`GameContent/UI/States/UIResourcePackInfoMenu.cs:
//! 15-210`) — the pack-info screen: the name/author/version header, the
//! divider, the wrapped description in a scrollable list and the centred
//! Back panel. W4-d84 row 6; `FrontScreen::ResourcePackInfo`.
//!
//! Deviations (docs/ANALYSIS.md "The Workshop/ResourcePack screens"):
//! - The description's word wrap is the port's greedy `wrap_text_with`
//!   (the bestiary twin), not `ChatManager.LayoutSnippets`' snippet walk;
//!   plain pack descriptions wrap identically.
//! - The gamepad snap points (`SetupGamepadPoints`, `:194-209`) are out of
//!   scope (§J.3).

use glam::Vec2;
use tr_render::SpriteRenderer;

use crate::assets;
use crate::resource_pack::ResourcePack;
use crate::ui::text::{self, font_width};
use crate::ui::uistate::{self, Style};
use crate::ui::{color, panel, workshop_kit as kit};

/// The screen's own state — `_pack` (`UIResourcePackInfoMenu.cs:19`) plus the
/// `_scrollbar.ViewPosition` the wheel/drag write, and the `_resourceMenu`
/// back target (`:17`). The back target is the exact parent selection screen
/// (the C# `SetState(this._resourceMenu)` restores the live instance — the
/// selection menu has no `OnActivate` re-list, so its unapplied pack edits
/// survive the info round-trip). Boxed: the target is another `FrontScreen`.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PackInfoState {
    /// `_pack` (`:19`) — the pack the info button named
    /// (`UIResourcePackSelectionMenu.Click_Info`, `:263-270`).
    pub pack: Option<ResourcePack>,
    /// `_scrollbar.ViewPosition` (`:131-136`) — the description list's offset.
    pub scroll: f32,
    /// `_resourceMenu` (`:17`) — the selection screen Back returns to.
    pub back: Option<Box<crate::app::flow::FrontScreen>>,
}

/// Every static rect of the screen (`BuildPage`, `:40-148`), resolved through
/// `uistate::resolve` in construction order. `desc_h` is the wrapped
/// description's content height (the `UIList`'s inner height,
/// `UIList.cs:120-128`); the scrollbar attaches only while it exceeds the
/// list view (`Recalculate`, `:150-168`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Layout {
    /// The 80%-wide, 500-max/300-min root (`:43-50`).
    pub root: [f32; 4],
    /// The container panel, `Height (-110, 1)` (`:51-55`).
    pub panel: [f32; 4],
    /// The 52 px header block (`:62-68`): name centred, author bottom-left,
    /// version bottom-right (`:69-90`).
    pub header: [f32; 4],
    /// The divider image — `Top 52`, `SetPadding(6)`, `Width (0, 1)`,
    /// `ScaleToFit` (`:91-100`).
    pub divider: [f32; 4],
    /// `_container` — `VAlign 1`, `Height (-74, 1)` of the panel inner
    /// (`:101-109`).
    pub container: [f32; 4],
    /// The description `UIList` (`:119-130`): `Width (0, 1)` detached /
    /// `(-25, 1)` attached (`:158/:164`), `PaddingRight 20`.
    pub list: [f32; 4],
    /// The scrollbar attach rect (`:131-135`): `Width 20`, `HAlign 1`,
    /// `Height (0, 1)` of the container.
    pub scrollbar: [f32; 4],
    /// The centred Back panel (`:137-142`): `Width (-10, 0.5)`, `Height 50`,
    /// `VAlign 1`, `HAlign 0.5`, `Top -45`.
    pub back: [f32; 4],
    /// Whether the scrollbar is attached (`Recalculate`'s `CanScroll`,
    /// `:154-165`).
    pub attached: bool,
}

/// The screen's layout for `desc_h` px of wrapped description. The root's
/// `MinWidth 300` (`:46`) rides `resolve`'s `min_w`.
pub(crate) fn layout(screen: Vec2, desc_h: f32) -> Layout {
    // `:43-50` — Width (0, 0.8), MaxWidth 500, MinWidth 300, Top 230,
    // Height (-230, 1), HAlign 0.5.
    let root = uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),
        Style::px(230.0),
        Style::new(0.0, 0.8),
        Style::new(-230.0, 1.0),
        0.5,
        0.0,
        300.0,
        0.0,
        500.0,
        [0.0; 4],
    );
    // `:51-55` — Width (0, 1), Height (-110, 1).
    let panel = kit::workshop_panel(root);
    let panel_inner = uistate::inner(panel, [12.0; 4]);
    // `:56-61` — the fill element; `:62-68` — the 52 px header block.
    let header = [panel_inner[0], panel_inner[1], panel_inner[2], 52.0];
    // `:91-100` — the divider at Top 52, padding 6, ScaleToFit: the image
    // fits the (width - 12) box keeping the sheet's aspect; the port draws
    // the sheet stretched to the box width at its own height (the 8x4
    // divider's aspect is flat — the same convention `char_select` uses).
    let divider = [
        panel_inner[0] + 6.0,
        panel_inner[1] + 52.0,
        panel_inner[2] - 12.0,
        4.0,
    ];
    // `:101-109` — _container: HAlign 0.5, VAlign 1, Width (0, 1),
    // Height (-74, 1).
    let container = uistate::resolve(
        panel_inner,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(0.0, 1.0),
        Style::new(-74.0, 1.0),
        0.5,
        1.0,
        0.0,
        0.0,
        panel_inner[2],
        [0.0; 4],
    );
    // `Recalculate` (`:150-168`): the scrollbar attaches while the content
    // exceeds the view; the list width drops 25 when attached (`:164`).
    let attached = kit::list_scrollbar_attaches(container[3], desc_h);
    let list_w = container[2] - if attached { 25.0 } else { 0.0 };
    let list = [container[0], container[1], list_w, container[3]];
    let scrollbar = kit::workshop_scrollbar(container);
    // `:137-142` — the Back panel of the ROOT (not the panel).
    let back = kit::workshop_back(root);
    Layout {
        root,
        panel,
        header,
        divider,
        container,
        list,
        scrollbar,
        back,
        attached,
    }
}

/// The wrapped description's content height — the `UIList`'s single wrapped
/// `UIText` child (`:110-128`): `WrappedTextBottomPadding = 0` (`:117`), so
/// the height is exactly the wrapped lines' total.
pub(crate) fn description_height(
    r: &SpriteRenderer,
    lookups: &assets::Lookups,
    pack: &ResourcePack,
    list_w: f32,
) -> f32 {
    // The list's `PaddingRight 20` (`:125`) shrinks the wrap width.
    let lines = kit::wrap_text(r, lookups, &pack.description, 1.0, list_w - 20.0, false);
    kit::wrapped_height(lookups, lines.len(), 1.0, false, 0.0)
}

/// The screen as `Recalculate` leaves it: the description height depends on
/// the list width, which depends on the attach branch — the C# converges in
/// one `Recalculate` pass because the wrap re-runs after the width flip; the
/// port resolves the detached width first, then re-measures if attached.
pub(crate) fn resolved_layout(
    r: &SpriteRenderer,
    lookups: &assets::Lookups,
    screen: Vec2,
    pack: &ResourcePack,
) -> Layout {
    let detached = layout(screen, 0.0);
    let desc_h = description_height(r, lookups, pack, detached.list[2]);
    let l = layout(screen, desc_h);
    if l.attached {
        // The attached list is 25 narrower — the wrap may grow a line.
        let desc_h2 = description_height(r, lookups, pack, l.list[2]);
        return layout(screen, desc_h2);
    }
    l
}

/// `UIResourcePackInfoMenu.Draw` (`:188-192` → `base.Draw` → the children's
/// `DrawSelf`s). `mouse` drives the Back panel's `FadedMouseOver`.
pub(crate) fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &PackInfoState,
    screen: Vec2,
    mouse: Vec2,
) {
    let Some(pack) = &state.pack else { return };
    let l = resolved_layout(r, lookups, screen, pack);
    // The container panel (`:51-55`), bg (33,43,79)*0.8.
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(kit::CLUSTER_PANEL_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.panel, color::tint(uistate::TEXT_PANEL_BORDER));

    // The header block (`:69-90`): the name (0.7 large, Gold, centred),
    // the author (0.9, bottom-left, Top -6) and the version (0.9, Silver,
    // bottom-right, Top -6).
    let name_w = match &lookups.death_font {
        Some(f) => font_width(f, &pack.name, 0.7),
        None => r.text_width(&pack.name, 0.7),
    };
    let name_x = l.header[0] + (l.header[2] - name_w) * 0.5;
    // `UIText.DrawSelf` large: Y -= 10*scale (`UIText.cs:134`).
    let name_y = l.header[1] - 10.0 * 0.7;
    text::draw_border_string(
        r, lookups, text::BorderFont::Death, name_x, name_y, 0.7,
        &[[1.0, 1.0]], color::tint([255, 215, 0, 255]), &pack.name, // Color.Gold
    );
    let author = kit::author_line(pack);
    let line_y = l.header[1] + l.header[3] - 16.0 * 0.9 - 6.0 - 2.0 * 0.9;
    text::draw_border_string(
        r, lookups, text::BorderFont::Ui, l.header[0], line_y, 0.9,
        &[[1.0, 1.0]], [1.0; 4], &author,
    );
    let version = kit::version_line(pack);
    let version_w = text::hud_text_width(r, lookups, &version, 0.9);
    text::draw_border_string(
        r, lookups, text::BorderFont::Ui,
        l.header[0] + l.header[2] - version_w, line_y, 0.9,
        &[[1.0, 1.0]], color::tint([192, 192, 192, 255]), &version, // Color.Silver
    );

    // The divider (`:91-100`): the sheet stretched over the padding-6 box.
    if let Some(sheet) = lookups.ui_divider {
        crate::ui::push_whole(r, lookups, &sheet, l.divider, [1.0; 4]);
    }

    // The description list (`:110-130`): the wrapped UIText at the list's
    // inner top, lifted by the scroll (`UIList.cs:181`), clipped to the
    // container (`UIList.cs:59`'s OverflowHidden — the port's row filter).
    let lines = kit::wrap_text(r, lookups, &pack.description, 1.0, l.list[2] - 20.0, false);
    let line_h = text::line_height(lookups);
    let mut ty = l.list[1] - state.scroll - 2.0; // UIText small: Y -= 2*scale
    for line in &lines {
        // The viewport filter: a line fully outside the container is skipped
        // (the scissor's draw-side equivalent).
        if ty + line_h > l.container[1] && ty < l.container[1] + l.container[3] {
            // HAlign 0.5 + TextOriginX 0.5 (`:112`, `UIText.cs:84`): centred.
            let lw = text::hud_text_width(r, lookups, line, 1.0);
            let tx = l.list[0] + (l.list[2] - 20.0 - lw) * 0.5;
            text::border_string(r, lookups, tx, ty, [1.0; 4], line);
        }
        ty += line_h;
    }

    // The scrollbar (`:131-136`, attached only while CanScroll).
    if l.attached {
        let desc_h = description_height(r, lookups, pack, l.list[2]);
        panel::draw_scrollbar(
            r, lookups, l.scrollbar, l.container[3], desc_h, state.scroll, false,
        );
    }

    // The Back panel (`:137-147`): 0.7 large, FadedMouseOver/Out.
    let back_text = crate::lang::text("UI.Back");
    let back_w = match &lookups.death_font {
        Some(f) => font_width(f, &back_text, 1.0),
        None => r.text_width(&back_text, 1.0),
    };
    let hover = crate::ui::in_rect(mouse, l.back);
    uistate::draw_text_panel(
        r,
        lookups,
        l.back,
        &uistate::TextPanelStyle {
            text_scale: 0.7,
            large: true,
            padding: 12.0,
            bg: uistate::TEXT_PANEL_BG,
            border: uistate::TEXT_PANEL_BORDER,
        },
        &back_text,
        back_w,
        hover,
    );
}

/// The screen's click table — `GoBackClick` (`:170-173`):
/// `Main.MenuUI.SetState(this._resourceMenu)`. The port returns the Back
/// click; the flow table reopens the selection menu. No sound here — the
/// C# `GoBackClick` plays none (the `FadedMouseOver` hover cue is the 12).
pub(crate) fn click_at(l: &Layout, mouse: Vec2) -> bool {
    crate::ui::in_rect(mouse, l.back)
}

/// `UIList.ScrollWheel` over the description list (`UIList.cs:107-114`):
/// the whole list rect scrolls (the inner list's `ContainsPoint` is true,
/// `UIList.cs:16-19`), the attached bar is its own target.
pub(crate) fn wheel_target(l: &Layout, mouse: Vec2) -> bool {
    crate::ui::in_rect(mouse, l.list) || (l.attached && crate::ui::in_rect(mouse, l.scrollbar))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> ResourcePack {
        ResourcePack {
            file_name: "test".into(),
            full_path: "test".into(),
            name: "Test Pack".into(),
            description: "A short description".into(),
            author: "Author".into(),
            version: crate::resource_pack::PackVersion { major: 1, minor: 2 },
            is_enabled: false,
            sorting_order: 0,
            is_compressed: false,
        }
    }

    /// `BuildPage` (`:43-50`): the root is 80% wide, 500 max / 300 min,
    /// Top 230, Height (-230, 1), centred.
    #[test]
    fn the_root_follows_the_buildpage_dimensions() {
        let l = layout(Vec2::new(1600.0, 900.0), 0.0);
        // 0.8 * 1600 = 1280 -> clamped to 500; x = (1600-500)/2 = 550.
        assert_eq!(l.root, [550.0, 230.0, 500.0, 670.0]);
        // The panel: Height (-110, 1) = 560.
        assert_eq!(l.panel, [550.0, 230.0, 500.0, 560.0]);
        // A narrow screen clamps to the 300 min (`:46`).
        let l = layout(Vec2::new(300.0, 900.0), 0.0);
        assert_eq!(l.root[2], 300.0);
        assert_eq!(l.root[0], 0.0);
    }

    /// `Recalculate` (`:150-168`): a short description detaches the bar and
    /// the list keeps the full width; a tall one attaches and drops 25.
    #[test]
    fn the_scrollbar_attach_branch_flips_the_list_width() {
        let short = layout(Vec2::new(1600.0, 900.0), 10.0);
        assert!(!short.attached);
        assert_eq!(short.list[2], short.container[2]);
        let tall = layout(Vec2::new(1600.0, 900.0), 10_000.0);
        assert!(tall.attached);
        assert_eq!(tall.list[2], tall.container[2] - 25.0);
    }

    /// `GoBackClick` (`:170-173`): only the Back panel is clickable.
    #[test]
    fn only_the_back_panel_answers_clicks() {
        let l = layout(Vec2::new(1600.0, 900.0), 0.0);
        let back_centre = Vec2::new(l.back[0] + l.back[2] * 0.5, l.back[1] + l.back[3] * 0.5);
        assert!(click_at(&l, back_centre));
        assert!(!click_at(&l, Vec2::new(l.panel[0] + 10.0, l.panel[1] + 10.0)));
    }

    /// The author/version lines ride `UI.Author`/`UI.Version`
    /// (`:76`, `:83`) with the pack's fields.
    #[test]
    fn the_header_lines_format_the_pack_fields() {
        crate::lang::init(tr_core::localization::GameCulture::English);
        let p = pack();
        assert_eq!(kit::author_line(&p), "by Author");
        assert_eq!(kit::version_line(&p), "Version 1.2");
    }
}
