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
//! (the [`GeneralIssueReporter`] below — the only non-Steam producer is
//! the resource-pack music failure, `LegacyAudioSystem.cs:114-119`) +
//! the Steam Workshop reporter (`SocialAPI.Workshop != null` only). The
//! port has no resource-pack pipeline, so the log is empty and the
//! `Workshop.ReportLogsInitialMessage` row draws (`PopulateLogs`'s empty
//! arm, `:147-160`) — the exact vanilla state with no packs and no Steam
//! (docs/research/112-doc-sweep-screens.md §3). **The page has no report
//! SUBMIT**: `UIReportsPage` is a read-only log viewer; the only "send"
//! in the C# is the SteamUGC publish (`WorkshopHelper.cs:247-318`,
//! `CreateItem`/`SubmitItemUpdate`), Steam-only and out of scope (§38).
//! The screen is reachable through `TR_UI=reports` (the port's preset;
//! the vanilla entries are the Workshop hub's Reports button,
//! `UIWorkshopHub.cs:295-300`, and the issue indicator's click,
//! `IssueReportsIndicator.cs:64-68`).

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

/// `GeneralIssueReporter` (`DataStructures/GeneralIssueReporter.cs:5-18`)
/// — `Main.IssueReporter` (`Main.cs:2169`, created `:10832`): an unbounded
/// append log. The only non-Steam producer is the resource-pack music
/// replacement failure (`LegacyAudioSystem.FindReplacementTrack`'s catch,
/// `LegacyAudioSystem.cs:114-119` — "A resource pack failed to load
/// {assetPath}!"); the port has no resource-pack pipeline, so the log
/// stays empty exactly as vanilla with no packs installed.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GeneralIssueReporter {
    reports: Vec<Report>,
}

impl GeneralIssueReporter {
    /// `AddReport` (`:9-12`) — `new IssueReport(text)` appends.
    pub fn add_report(&mut self, text: String) {
        self.reports.push(Report { text });
    }

    /// `GetReports` (`:14-17`) — the live list.
    pub fn get_reports(&self) -> &[Report] {
        &self.reports
    }
}

/// `WorkshopIssueReporter._maxReports` (`Social/Base/WorkshopIssueReporter.
/// cs:10`) — the Steam reporter's 1000-entry ring: `AddReport` appends then
/// drops from the front while over the cap (`:18-26`). Steam-only
/// (`SocialAPI.Workshop != null`, `Main.cs:52356-52359`); the port's
/// `Workshop == null` state never constructs one — the shape is pinned
/// here for the census.
pub const WORKSHOP_MAX_REPORTS: usize = 1000;

/// The Workshop reporter's capped append (`AddReport`, `:18-26`) over a
/// caller-owned log — the ring rule without the Steam events.
pub fn workshop_add_report(reports: &mut Vec<Report>, text: String) {
    reports.push(Report { text });
    while reports.len() > WORKSHOP_MAX_REPORTS {
        reports.remove(0);
    }
}

/// `Main.OpenReportsMenu`'s reporter merge (`Main.cs:52352-52363`):
/// `Main.IssueReporter` first, then `SocialAPI.Workshop.IssueReporter`
/// when Steam is live; `PopulateLogs` orders the merged rows by
/// `timeReported` (a stable sort — the append order already is it).
/// The port passes only the general log (the `Workshop == null` arm).
pub fn merged_reports(general: &[Report], workshop: Option<&[Report]>) -> Vec<Report> {
    let mut out = general.to_vec();
    if let Some(w) = workshop {
        out.extend_from_slice(w);
    }
    out
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
    // `UIScrollbar.SetView` (`:74`) clamps the view into `[0, maxViewSize]`
    // before `CanScroll` (`:49`) compares it with the max: a short log
    // (content < view) pins the view to the content, so the bar detaches.
    let view = view_h.clamp(0.0, content_h);
    content_h != view
}

// --- The issue indicator (`IssueReportsIndicator.cs:10-80`) ------------------
//
// The consumer-side half of the reporter pipeline (W4-d85 FINAL): the
// menu-tail button `Main.DrawMenu` draws (`Main.cs:51886`, right before
// `DrawCursor`) that opens this page. `AttemptLettingPlayerKnow` (`:20-25`)
// is called by the resource-pack music failure (`LegacyAudioSystem.cs:118`)
// and the Steam Workshop reporter's notify event
// (`WorkshopSocialModule.cs:42-46`); with neither pipeline the port's
// indicator never shows — the exact vanilla state on a pack-free, no-Steam
// install. The state machine is ported so a future producer (or a test)
// drives the same slide-in / hover / click-to-open flow.

/// `IssueReportsIndicator` (`IssueReportsIndicator.cs:12-18`) — the two
/// private fields; the two `Asset<Texture2D>` requests (`Setup`, `:39-43`)
/// resolve through the atlas Lookups instead.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IssueIndicator {
    /// `_displayUpPercent` (`:12`) — the slide-in/out accumulator.
    pub display_up_percent: f32,
    /// `_shouldBeShowing` (`:14`).
    pub should_be_showing: bool,
}

impl IssueIndicator {
    /// `AttemptLettingPlayerKnow` (`:20-25`): `Setup()` + the show latch.
    /// The `SoundEngine.PlaySound(SoundID.DD2_ExplosiveTrapExplode)` cue is
    /// the caller's (`App::issue_report_notify` — the audio system lives on
    /// the App, not here).
    pub fn attempt_letting_player_know(&mut self) {
        self.should_be_showing = true;
    }

    /// `Hide` (`:27-31`): the latch clears AND the percent snaps to 0 (no
    /// slide-out when explicitly hidden — the Workshop hub's Logs button
    /// and the indicator's own click call this).
    pub fn hide(&mut self) {
        self.should_be_showing = false;
        self.display_up_percent = 0.0;
    }

    /// `Draw`'s first two lines (`:47-48`): the percent walks ±1 per DRAWN
    /// frame (`shouldBeShowing.ToDirectionInt()` — +1 showing, −1 hiding,
    /// `Utils.cs:2010-2017`), clamped 0..1. Returns whether the button draws
    /// this frame (`:49-52`'s zero early-out).
    pub fn tick_draw(&mut self) -> bool {
        let dir = if self.should_be_showing { 1.0 } else { -1.0 };
        self.display_up_percent = (self.display_up_percent + dir).clamp(0.0, 1.0);
        self.display_up_percent != 0.0
    }
}

/// The button's centre (`Draw`, `:54-55`): `ScreenSize + (40, -80)` lerped
/// to `+ (-80, 0)` by the display percent — parked 40 px off the right edge,
/// slid in to `screenWidth - 40, screenHeight - 80`.
pub fn indicator_center(screen: Vec2, percent: f32) -> Vec2 {
    let parked = screen + Vec2::new(40.0, -80.0);
    parked.lerp(parked + Vec2::new(-80.0, 0.0), percent)
}

/// The hover/click box (`Draw`, `:59`): `Utils.CenteredRectangle(center,
/// size)` (`Utils.cs:947-950`) — the int-truncated top-left, the size as-is.
/// `ContainsPoint` is `Rectangle.Contains` on the truncated mouse point
/// (`Main.MouseScreen.ToPoint()`).
pub fn indicator_hit(center: Vec2, size: Vec2, mouse: Vec2) -> bool {
    let x = (center.x - size.x * 0.5) as i32;
    let y = (center.y - size.y * 0.5) as i32;
    let (mx, my) = (mouse.x as i32, mouse.y as i32);
    mx >= x && mx < x + size.x as i32 && my >= y && my < y + size.y as i32
}

/// `Draw` (`:45-79`): the button at the lerped centre (origin = half the
/// frame, scale 1, white), the outline over it while hovered, and the
/// `UI.IssueReporterHasThingsToShow` mouse text (`:62-63`). The click arm
/// (`:64-68` — `OpenUI` + `Hide`) is the App's (`issue_indicator_click`):
/// this draw only reports the hover. Returns the hover flag.
pub fn draw_indicator(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    percent: f32,
    mouse: Vec2,
    pulse: super::UiPulse,
) -> bool {
    let Some(sheet) = lookups.ui_issue_button else {
        return false;
    };
    let size = Vec2::new(sheet.width as f32, sheet.height as f32);
    let center = indicator_center(screen, percent);
    let hovered = indicator_hit(center, size, mouse);
    super::push_whole(
        r,
        lookups,
        &sheet,
        [center.x - size.x * 0.5, center.y - size.y * 0.5, size.x, size.y],
        [1.0; 4],
    );
    if hovered {
        if let Some(outline) = lookups.ui_issue_button_outline {
            let os = Vec2::new(outline.width as f32, outline.height as f32);
            super::push_whole(
                r,
                lookups,
                &outline,
                [center.x - os.x * 0.5, center.y - os.y * 0.5, os.x, os.y],
                [1.0; 4],
            );
        }
        super::tooltip::draw_mouse_text(
            r,
            lookups,
            mouse,
            &lang::text("UI.IssueReporterHasThingsToShow"),
            screen,
            pulse,
        );
    }
    hovered
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

    /// `GeneralIssueReporter.AddReport`/`GetReports`
    /// (`GeneralIssueReporter.cs:9-17`): the unbounded append log;
    /// `OpenReportsMenu`'s merge passes the general rows first, the
    /// Workshop rows after (`Main.cs:52354-52359`).
    #[test]
    fn the_general_reporter_appends_and_merges() {
        let mut r = GeneralIssueReporter::default();
        assert!(r.get_reports().is_empty());
        r.add_report("A resource pack failed to load Music\\Music_1.ogg!".into());
        r.add_report("second".into());
        assert_eq!(r.get_reports().len(), 2);
        assert_eq!(r.get_reports()[0].text, "A resource pack failed to load Music\\Music_1.ogg!");
        // The merge: general first, workshop after; the null-Steam arm
        // passes only the general log.
        let workshop = vec![Report { text: "w1".into() }];
        let merged = merged_reports(r.get_reports(), Some(&workshop));
        assert_eq!(merged.len(), 3);
        assert_eq!(merged[2].text, "w1");
        assert_eq!(merged_reports(r.get_reports(), None), r.get_reports());
    }

    /// `IssueReportsIndicator`'s state machine (`IssueReportsIndicator.cs:
    /// 20-31/:47-55`): the notify latches the show, the percent walks ±1 per
    /// drawn frame clamped 0..1, `Hide` snaps to 0, and the centre lerps
    /// from the parked off-screen point to the slid-in one.
    #[test]
    fn the_indicator_slides_in_and_out_per_drawn_frame() {
        let mut ind = IssueIndicator::default();
        // Hidden: the percent walks −1, clamps at 0, nothing draws.
        assert!(!ind.tick_draw());
        assert_eq!(ind.display_up_percent, 0.0);
        // Notify: the latch sets, the percent climbs +1 per frame, clamped.
        ind.attempt_letting_player_know();
        assert!(ind.should_be_showing);
        assert!(ind.tick_draw());
        assert_eq!(ind.display_up_percent, 1.0);
        assert!(ind.tick_draw(), "clamped at 1, still draws");
        // The centre: parked at screen + (40, -80), slid to + (-40, -80).
        let screen = Vec2::new(1600.0, 900.0);
        assert_eq!(indicator_center(screen, 0.0), Vec2::new(1640.0, 820.0));
        assert_eq!(indicator_center(screen, 1.0), Vec2::new(1560.0, 820.0));
        assert_eq!(indicator_center(screen, 0.5), Vec2::new(1600.0, 820.0));
        // Hide: the latch clears AND the percent snaps (no slide-out).
        ind.hide();
        assert!(!ind.should_be_showing);
        assert_eq!(ind.display_up_percent, 0.0);
        assert!(!ind.tick_draw());
        // A notify after a hide re-shows; a plain latch-clear slides OUT
        // one percent per frame (the C# has no other clear path): 1 → 0.
        ind.attempt_letting_player_know();
        ind.tick_draw();
        assert_eq!(ind.display_up_percent, 1.0);
        ind.should_be_showing = false;
        assert!(!ind.tick_draw(), "the −1 step lands at 0 — nothing draws");
        assert_eq!(ind.display_up_percent, 0.0);
    }

    /// The indicator's hover box (`Draw`, `:59` — `Utils.CenteredRectangle`
    /// + `Rectangle.Contains` on the truncated mouse point).
    #[test]
    fn the_indicator_hit_is_the_centered_int_rect() {
        let center = Vec2::new(1560.0, 820.0);
        let size = Vec2::new(34.0, 34.0);
        // The box: (1543, 803)..(1577, 837) — int-truncated top-left.
        assert!(indicator_hit(center, size, Vec2::new(1560.0, 820.0)));
        assert!(indicator_hit(center, size, Vec2::new(1543.0, 803.0)), "the left/top edge is inclusive");
        assert!(!indicator_hit(center, size, Vec2::new(1577.0, 820.0)), "the right edge is exclusive");
        assert!(!indicator_hit(center, size, Vec2::new(1560.0, 837.0)), "the bottom edge is exclusive");
        // The mouse point truncates (`Main.MouseScreen.ToPoint()`).
        assert!(indicator_hit(center, size, Vec2::new(1543.9, 803.9)));
        assert!(!indicator_hit(center, size, Vec2::new(1542.9, 803.0)));
    }

    /// `WorkshopIssueReporter.AddReport`'s 1000-entry ring
    /// (`WorkshopIssueReporter.cs:10/:18-26`): the append drops from the
    /// front while over the cap.
    #[test]
    fn the_workshop_ring_caps_at_1000() {
        let mut reports = Vec::new();
        for i in 0..WORKSHOP_MAX_REPORTS {
            workshop_add_report(&mut reports, format!("r{i}"));
        }
        assert_eq!(reports.len(), 1000);
        assert_eq!(reports[0].text, "r0");
        workshop_add_report(&mut reports, "overflow".into());
        assert_eq!(reports.len(), 1000, "the cap holds");
        assert_eq!(reports[0].text, "r1", "the front dropped");
        assert_eq!(reports[999].text, "overflow");
    }
}
