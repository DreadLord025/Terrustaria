//! The reports page (`UIReportsPage`,
//! `GameContent/UI/States/UIReportsPage.cs:16-248`) — W4-d85 row 10. The
//! Workshop issue-log screen `Main.OpenReportsMenu` opens
//! (`Main.cs:52352-52363`): the 0.8-wide container (MaxWidth 500 /
//! MinWidth 300) at Top 230, the `(33,43,79)*0.8` panel, the 28 px
//! "Reports Page" header, the highlighted log area with the wrapped
//! report rows + dividers (ListPadding 40), the scrollbar that attaches
//! only when the content overflows (`Recalculate`, `:186-204`) and starts
//! at the bottom (`GoToBottom`, `:124`), and the fat Back panel.
//!
//! The reporters (`IProvideReports.GetReports`) are `Main.IssueReporter`
//! + the Workshop reporter; the port has no issue-report pipeline, so the
//! log is empty and the `Workshop.ReportLogsInitialMessage` row draws
//! (`PopulateLogs`'s empty arm, `:147-160`) — documented deviation
//! (docs/ANALYSIS.md "Reports page (W4-d85)"). The screen is reachable
//! through `TR_UI=reports` (the port's preset; the vanilla entry is the
//! Workshop hub's Reports button, d84's lane).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text;
use super::uistate::{self, Style};
use super::{color, in_rect, panel};
use crate::assets::Lookups;
use crate::lang;

/// One `IssueReport` (`DataStructures/IssueReport.cs:5-16`) — the text
/// plus the sort key (`timeReported`; the port keeps the insertion order,
/// which IS the time order for a live log).
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub text: String,
}

/// `PopulateLogs` (`UIReportsPage.cs:142-184`): the reports ordered by
/// `timeReported` (a stable sort — the port's Vec order is already the
/// append order). Empty input draws the initial-message row instead.
pub fn sorted_reports(reports: &[Report]) -> Vec<&Report> {
    reports.iter().collect()
}

/// `ListPadding = 40f` (`UIReportsPage.cs:112`).
pub const LIST_PADDING: f32 = 40.0;

// --- Geometry (`BuildPage`, `UIReportsPage.cs:48-136`) ------------------------

/// The outer container: `Width (0, 0.8)`, `MaxWidth 500`, `MinWidth 300`,
/// `Top 230`, `Height (-230, 1)`, `HAlign 0.5` (`:51-57`).
pub fn container_rect(screen: Vec2) -> [f32; 4] {
    uistate::resolve(
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
    )
}

/// The `UIPanel`: `Width (0, 1)`, `Height (-110, 1)` (`:59-63`).
pub fn panel_rect(container: &[f32; 4]) -> [f32; 4] {
    [container[0], container[1], container[2], container[3] - 110.0]
}

/// The header strip: `Width (0, 1)`, `Height 28` (`:70-76`), inside the
/// panel's 12 px padding.
pub fn header_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(*panel, [12.0; 4]);
    [inner[0], inner[1], inner[2], 28.0]
}

/// The log container: `HAlign 0.5, VAlign 1, Width (0, 1),
/// Height (-40, 1), Top -2` (`:81-90`), inside the panel's inner rect.
pub fn log_rect(panel: &[f32; 4]) -> [f32; 4] {
    let inner = uistate::inner(*panel, [12.0; 4]);
    [inner[0], inner[1] + 40.0 - 2.0, inner[2], inner[3] - 40.0]
}

/// The `UIList` inside the log area: `Width (-10, 1)` when the scrollbar
/// is detached, `(-25, 1)` when attached (`Recalculate`, `:194`/`:200`),
/// `Height (0, 1)`, `PaddingRight 20` (`:104-111`). The list's own
/// `OverflowHidden` clips at its outer rect (`UIList.cs:59`).
pub fn list_rect(log: &[f32; 4], scrollbar_attached: bool) -> [f32; 4] {
    let shrink = if scrollbar_attached { 25.0 } else { 10.0 };
    [log[0], log[1], log[2] - shrink, log[3]]
}

/// The attached scrollbar: `Height (0, 1)`, `HAlign 1` of the log
/// container (`:118-121`), width 20.
pub fn scrollbar_rect(log: &[f32; 4]) -> [f32; 4] {
    [log[0] + log[2] - 20.0, log[1], 20.0, log[3]]
}

/// The Back panel: `Width (-10, 0.5)`, `Height 50`, `VAlign 1`,
/// `HAlign 0.5`, `Top -45` (`:125-130`).
pub fn back_rect(container: &[f32; 4]) -> [f32; 4] {
    let w = container[2] * 0.5 - 10.0;
    [
        container[0] + container[2] * 0.5 - w * 0.5,
        container[1] + container[3] - 45.0 - 50.0,
        w,
        50.0,
    ]
}

/// One report row's height: the wrapped `UIText` at scale 1 with
/// `WrappedTextBottomPadding = 0` (`:163-172`) — the line count times the
/// Mouse font's line spacing (`UIText.InternalSetText`, `UIText.cs:
/// 178-183`), plus the divider child's own height (the `UIImage`'s
/// `Height = asset.Height()`, `:174-181` — `UI/Divider` is 4 px tall).
/// The row's `MinHeight` is the text height (`UIText.cs:189`); the
/// divider appends BELOW the text inside the same element (its `VAlign
/// 1` rides the text's own height, so the row's outer height is
/// `max(textH, dividerH)` = textH for any non-empty wrap).
pub fn row_height(lines: usize, lookups: &Lookups) -> f32 {
    (lines.max(1) as f32) * text::line_height(lookups)
}

/// The list content height (`UIList.RecalculateChildren`,
/// `UIList.cs:116-128`): the leading empty `UIElement` (0 px, `:114`),
/// each row + 40 padding, the trailing empty element (`:183`) — with the
/// single-item special case dropping the padding. The empty log's
/// initial-message row counts as one item between the two empties.
pub fn content_height(row_heights: &[f32]) -> f32 {
    // items = [empty(0)] + rows + [empty(0)]; N = rows + 2.
    let n = row_heights.len() + 2;
    let pad = if n == 1 { 0.0 } else { LIST_PADDING };
    let mut h = 0.0;
    for rh in std::iter::once(&0.0)
        .chain(row_heights.iter())
        .chain(std::iter::once(&0.0))
    {
        h += rh + pad;
    }
    h
}

/// `UIScrollbar.GoToBottom` (`UIScrollbar.cs:80-83`): `ViewPosition =
/// MaxViewSize - ViewSize` — the open scrolls to the newest report.
pub fn go_to_bottom(view_h: f32, content_h: f32) -> f32 {
    (content_h - view_h).max(0.0)
}

/// Whether the scrollbar attaches (`Recalculate`'s `CanScroll` arm,
/// `:190-201` — `UIScrollbar.CanScroll` is `maxView != view`).
pub fn scrollbar_attaches(view_h: f32, content_h: f32) -> bool {
    content_h != view_h
}

// --- Hits --------------------------------------------------------------------

/// The Back panel under the pointer (`GoBackClick`, `:206-211`).
pub fn back_hit(screen: Vec2, mouse: Vec2) -> bool {
    in_rect(mouse, back_rect(&container_rect(screen)))
}

/// The wheel target: the list rect (the `UIInnerList.ContainsPoint`
/// always-true, `UIList.cs:16-19`) or the attached scrollbar.
pub fn wheel_target(screen: Vec2, mouse: Vec2, view_h: f32, content_h: f32) -> bool {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    let log = log_rect(&panel);
    let attached = scrollbar_attaches(view_h, content_h);
    let list = list_rect(&log, attached);
    in_rect(mouse, list) || (attached && in_rect(mouse, scrollbar_rect(&log)))
}

// --- Draw --------------------------------------------------------------------

/// The panel background — `new Color(33, 43, 79) * 0.8f` (`:62`).
const PANEL_BG: [u8; 4] = [26, 34, 63, 204];
/// The log highlight — `Color.LightGray * 0.5f` (`:102`).
const LOG_HIGHLIGHT: [u8; 4] = [105, 105, 105, 127];
/// The empty-log message tint — `Color.Gray` (`:158`).
const GRAY: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

/// `UIReportsPage.Draw` (the port draws over the front-end backdrop —
/// the state is a `Main.MenuUI` screen, `Main.cs:52361-52362`).
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    reports: &[Report],
    scroll: f32,
    mouse: Vec2,
) {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, panel, color::tint(PANEL_BG));
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );

    // The header: `UIText("UI.ReportsPage", 0.7, large)`, HAlign 0.5
    // (`:77-80`) — centred in the 28 px strip, lifted 10*0.7 (large).
    let header = header_rect(&panel);
    let label = lang::text("UI.ReportsPage");
    let w = match &lookups.death_font {
        Some(f) => text::font_width(f, &label, 0.7),
        None => r.text_width(&label, 0.7),
    };
    text::draw_border_string(
        r,
        lookups,
        text::BorderFont::Death,
        header[0] + (header[2] - w) * 0.5,
        header[1] - 10.0 * 0.7,
        0.7,
        &[[1.0, 1.0]],
        [1.0; 4],
        &label,
    );

    let log = log_rect(&panel);
    // The `CategoryPanelHighlight` slice behind the log (`:92-103`):
    // `Top 2`, full width/height, `LightGray * 0.5`.
    let highlight = [log[0], log[1] + 2.0, log[2], log[3]];
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_highlight,
        highlight,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(LOG_HIGHLIGHT),
    );

    // The rows: wrap each report at the list's inner width (the 20 px
    // PaddingRight, `:110`), then walk the lattice from the scroll.
    let rows = sorted_reports(reports);
    let provisional_list = list_rect(&log, false);
    let wrap_w = provisional_list[2] - 20.0;
    let wrapped: Vec<Vec<String>> = if rows.is_empty() {
        vec![wrap_lines(r, lookups, &lang::text("Workshop.ReportLogsInitialMessage"), 1.0, wrap_w)]
    } else {
        rows.iter()
            .map(|rep| wrap_lines(r, lookups, &rep.text, 1.0, wrap_w))
            .collect()
    };
    let heights: Vec<f32> = wrapped.iter().map(|lines| row_height(lines.len(), lookups)).collect();
    let content = content_height(&heights);
    let attached = scrollbar_attaches(log[3], content);
    let list = list_rect(&log, attached);

    r.push_clip(list);
    // The lattice: the leading empty element (0 + 40), then the rows.
    let mut y = list[1] + LIST_PADDING - scroll;
    for (i, lines) in wrapped.iter().enumerate() {
        let h = heights[i];
        if y + h >= list[1] && y <= list[1] + list[3] {
            let tint = if rows.is_empty() { GRAY } else { [1.0; 4] };
            // `TextOriginX = 0.5` for the empty message (`:157`), 0 for
            // the reports (`:171`); the small-font draw lifts 2*scale
            // (`UIText.cs:138`).
            let mut line_y = y;
            for line in lines {
                let lw = text::hud_text_width(r, lookups, line, 1.0);
                let x = if rows.is_empty() {
                    list[0] + (list[2] - 20.0 - lw) * 0.5
                } else {
                    list[0]
                };
                text::border_string(r, lookups, x, line_y - 2.0, tint, line);
                line_y += text::line_height(lookups);
            }
            // The divider child (`UIImage`, VAlign 1, ScaleToFit,
            // `:174-181`) — at the row's bottom, full inner width.
            if !rows.is_empty() {
                if let Some(sheet) = lookups.ui_divider {
                    super::push_whole(
                        r,
                        lookups,
                        &sheet,
                        [list[0], y + h - sheet.height as f32, list[2] - 20.0, sheet.height as f32],
                        [1.0; 4],
                    );
                }
            }
        }
        y += h + LIST_PADDING;
    }
    r.pop_clip();

    if attached {
        panel::draw_scrollbar(r, lookups, scrollbar_rect(&log), log[3], content, scroll, false);
    }

    // The Back panel (`UI.Back`, 0.7 large, the faded pair, `:125-134`).
    let back = back_rect(&container);
    let label = lang::text("UI.Back");
    let style = super::cards::fat_button_style();
    let w = text::hud_text_width(r, lookups, &label, style.text_scale);
    uistate::draw_text_panel(r, lookups, back, &style, &label, w, in_rect(mouse, back));
}

/// The renderer-free line count of [`wrap_lines`] — the wheel path's
/// content-height read (the retail font's metrics when loaded, the
/// built-in bitmap's 8 px-per-char advance otherwise — the
/// `SpriteRenderer::text_width` fallback at `font_glyph_px = 8`,
/// `app/loop.rs:71`).
pub(crate) fn wrap_line_count(text_in: &str, max_w: f32, lookups: &Lookups) -> usize {
    let width = |s: &str| -> f32 {
        match &lookups.ui_font {
            Some(f) => text::font_width(f, s, 1.0),
            None => s.chars().count() as f32 * 8.0,
        }
    };
    let mut lines = 0usize;
    let mut current_w = 0.0f32;
    let mut has_word = false;
    for word in text_in.split_whitespace() {
        let ww = width(word);
        let space = if has_word { width(" ") } else { 0.0 };
        if has_word && current_w + space + ww > max_w {
            lines += 1;
            current_w = ww;
        } else {
            current_w += space + ww;
        }
        has_word = true;
    }
    lines + 1
}

/// `UIText.IsWrapped`'s word wrap (`ChatManager.LayoutSnippets` at the
/// inner width) — the bestiary's greedy word wrap shape.
fn wrap_lines(
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `BuildPage`'s geometry at 1600x900, hand-derived from the C#.
    #[test]
    fn the_layout_matches_the_source_geometry() {
        let screen = Vec2::new(1600.0, 900.0);
        let c = container_rect(screen);
        // 0.8 * 1600 = 1280 clamped to MaxWidth 500; centred.
        assert_eq!(c, [550.0, 230.0, 500.0, 670.0]);
        let p = panel_rect(&c);
        assert_eq!(p, [550.0, 230.0, 500.0, 560.0]);
        assert_eq!(header_rect(&p), [562.0, 242.0, 476.0, 28.0]);
        // The log: VAlign 1 of the 536 inner, Height -40, Top -2:
        // y = 242 + 536 - 496 - 2 = 280.
        assert_eq!(log_rect(&p), [562.0, 280.0, 476.0, 496.0]);
        // Detached list -10, attached -25.
        assert_eq!(list_rect(&log_rect(&p), false), [562.0, 280.0, 466.0, 496.0]);
        assert_eq!(list_rect(&log_rect(&p), true), [562.0, 280.0, 451.0, 496.0]);
        assert_eq!(scrollbar_rect(&log_rect(&p)), [1018.0, 280.0, 20.0, 496.0]);
        // Back: W 240 centred; y = 230 + 670 - 95 = 805.
        assert_eq!(back_rect(&c), [680.0, 805.0, 240.0, 50.0]);
        // MinWidth 300 on a narrow screen.
        assert_eq!(container_rect(Vec2::new(320.0, 480.0))[2], 300.0);
    }

    /// The content lattice: two empty spacer elements + the rows, each
    /// followed by the 40 px padding (`UIList.cs:119-127`).
    #[test]
    fn the_content_height_counts_the_spacers() {
        // No reports: the initial-message row (one wrapped line, the
        // bitmap fallback's 8 px) between the two empties.
        let lookups = Lookups::default();
        let h = row_height(1, &lookups);
        assert_eq!(h, 8.0);
        // items = [0, 8, 0], N=3 → 3*40 + 8 = 128.
        assert_eq!(content_height(&[h]), 128.0);
        // Two 2-line reports: [0, 16, 16, 0] → 4*40 + 32 = 192.
        assert_eq!(content_height(&[16.0, 16.0]), 192.0);
    }

    /// `GoToBottom` (`:124`) and the attach rule (`Recalculate`,
    /// `:186-204`).
    #[test]
    fn the_scrollbar_starts_at_the_bottom_and_attaches_on_overflow() {
        assert_eq!(go_to_bottom(496.0, 1000.0), 504.0);
        assert_eq!(go_to_bottom(496.0, 128.0), 0.0);
        assert!(!scrollbar_attaches(496.0, 496.0));
        assert!(scrollbar_attaches(496.0, 1000.0));
        // A short log does not attach — the list keeps the -10 width.
        assert!(!scrollbar_attaches(496.0, 128.0));
    }

    /// The Back hit and the wheel target.
    #[test]
    fn the_back_and_wheel_hits() {
        let screen = Vec2::new(1600.0, 900.0);
        assert!(back_hit(screen, Vec2::new(800.0, 820.0)));
        assert!(!back_hit(screen, Vec2::new(800.0, 700.0)));
        // A short log: the list rect scrolls, the bar is detached.
        assert!(wheel_target(screen, Vec2::new(700.0, 400.0), 496.0, 128.0));
        assert!(!wheel_target(screen, Vec2::new(1030.0, 400.0), 496.0, 128.0));
        // Overflowing: the bar attaches and takes the wheel.
        assert!(wheel_target(screen, Vec2::new(1030.0, 400.0), 496.0, 1000.0));
    }
}
