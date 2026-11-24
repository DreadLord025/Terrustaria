//! The secret-seeds list (`UIWorldCreationAdvancedSecretSeedsList`,
//! `GameContent/UI/States/UIWorldCreationAdvancedSecretSeedsList.cs:17-459`)
//! — W4-d85 row 12. The Advanced tab's twin root (500 wide, Top 202,
//! MaxHeight 400, the -102 panel), the 40 px seed rows sorted by
//! `TextThatWasUsedToUnlock` (`CustomSort`, `:205-217`), the 56 px
//! description panel and the fat Apply panel. A row click toggles the
//! seed (`ClickSecretSeed`, `:219-236`): enabling adds the text to the
//! draft's entered list, disabling removes it; the enabled rows draw the
//! `IconRandomSeed` glow ring at the top-right (`DrawGlowRing`, `:182-203`
//! — the dead-code sine arms collapse to rotation 0, scale 1).
//!
//! The row set is `SecretSeedsTracker.SeedsForInterface` — the seeds the
//! player has unlocked (the settings-persisted tracker,
//! `SecretSeedsTracker.cs:21-40`). The port derives it from the draft's
//! entered + parked texts (no settings persistence — documented
//! deviation, docs/ANALYSIS.md "Advanced world creation (W4-d85)").
//!
//! Deviations: the enable particle burst (`SpawnParticles`, `:238-255`)
//! is not drawn; the gamepad snap points are out of scope (§J.3); the
//! `MouseOverSeed` held-button re-click (`:174-177`) is not ported (the
//! port's click fires on the press edge only).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text;
use super::uistate::{self, Style};
use super::world_creation::NewWorldDraft;
use super::{color, in_rect, panel};
use crate::assets::Lookups;
use crate::lang;

/// One row — a `WorldGen.SecretSeed` the interface tracks: the unlock
/// text (`TextThatWasUsedToUnlock`), the localization key
/// (`SecretSeedDescription.{id}`) and the enabled state.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedRow {
    /// The registered id (`MICRO_SEED_ORDER`'s name).
    pub id: &'static str,
    /// `TextThatWasUsedToUnlock` — the alnum+space-filtered input
    /// (`WorldGen.cs:522-523`).
    pub unlock_text: String,
    /// `Enabled` — the draft's entered list membership.
    pub enabled: bool,
}

/// `SecretSeedDescription.{id}` — the row's description key
/// (`WorldGen.cs:3022-3058`'s registration strings).
pub fn description_key(id: &str) -> String {
    format!("SecretSeedDescription.{id}")
}

/// The interface rows from the draft's lists — the port's
/// `SecretSeedsTracker.SeedsForInterface` stand-in: every distinct
/// registered seed the entered or parked texts unlock, sorted by the
/// unlock text (`CustomSort`, `:205-217` — an ordinal string compare).
pub fn interface_rows(draft: &NewWorldDraft) -> Vec<SeedRow> {
    let mut rows: Vec<SeedRow> = Vec::new();
    let mut push = |text: &str, enabled: bool| {
        let Some(id) = tr_core::seeds::check_input_for_secret_seed(text) else {
            return;
        };
        // `TextThatWasUsedToUnlock` = `Regex.Replace(input,
        // "[^a-zA-Z0-9 ]+", "")` (`WorldGen.cs:522`).
        let unlock_text: String = text
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
            .collect();
        if let Some(existing) = rows.iter_mut().find(|r| r.id == id) {
            // An entered text beats a parked one (the C#'s Distinct keeps
            // the first — the entered list is walked first here).
            existing.enabled |= enabled;
        } else {
            rows.push(SeedRow { id, unlock_text, enabled });
        }
    };
    for text in &draft.secret_texts {
        push(text, true);
    }
    for text in &draft.disabled_secret_texts {
        push(text, false);
    }
    rows.sort_by(|a, b| a.unlock_text.cmp(&b.unlock_text));
    rows
}

/// `ListPadding = 5f` (`:90`).
pub const LIST_PADDING: f32 = 5.0;
/// The row height (`:106` — `Height = 40`).
pub const ROW_H: f32 = 40.0;
/// The row stride.
pub const ROW_PITCH: f32 = ROW_H + LIST_PADDING;

// --- Geometry (`BuildPage`, `:48-131`) ----------------------------------------

/// The root: the Advanced tab's twin (`:53-62`) — MaxHeight 400 always.
pub fn root_rect(screen: Vec2) -> [f32; 4] {
    super::world_creation_advanced::root_rect(screen)
}

/// The panel: `Width 100%`, `Height (-102, 1)`, padding 0 (`:64-70`).
pub fn panel_rect(root: &[f32; 4]) -> [f32; 4] {
    [root[0], root[1], root[2], root[3] - 102.0]
}

/// The list container: `Top 4`, `Width (-20, 1)`, `Left 2`,
/// `Height (-60, 1)`, `HAlign 0.5`, `PaddingTop 8 / PaddingBottom 12`
/// (`:73-86` — `num = 56`, `num2 = 4`). The field carries the INNER rect.
pub fn list_container_inner(panel: &[f32; 4]) -> [f32; 4] {
    let outer = uistate::resolve(
        *panel,
        Style::px(2.0),
        Style::px(4.0),
        Style::new(-20.0, 1.0),
        Style::new(-60.0, 1.0),
        0.5,
        0.0,
        0.0,
        0.0,
        panel[2],
        [0.0, 8.0, 0.0, 12.0],
    );
    uistate::inner(outer, [0.0, 8.0, 0.0, 12.0])
}

/// The `UIList`: `Width (0, 1)` detached / `(-25, 1)` attached
/// (`Recalculate`, `:343-361`), `Height (0, 1)`.
pub fn list_rect(container_inner: &[f32; 4], attached: bool) -> [f32; 4] {
    let shrink = if attached { 25.0 } else { 0.0 };
    [
        container_inner[0],
        container_inner[1],
        container_inner[2] - shrink,
        container_inner[3],
    ]
}

/// The attached scrollbar: `Height (0, 1)`, `HAlign 1` of the container
/// (`:93-96`), width 20.
pub fn scrollbar_rect(container_inner: &[f32; 4]) -> [f32; 4] {
    [
        container_inner[0] + container_inner[2] - 20.0,
        container_inner[1],
        20.0,
        container_inner[3],
    ]
}

/// The row rect at `index` — the 40+5 lattice, `Width (0, 1)` of the list.
pub fn row_rect(list: &[f32; 4], index: usize) -> [f32; 4] {
    [list[0], list[1] + index as f32 * ROW_PITCH, list[2], ROW_H]
}

/// The list content height (`UIList.cs:119-127`): N*(40+5); one row 40.
pub fn content_height(rows: usize) -> f32 {
    match rows {
        0 => 0.0,
        1 => ROW_H,
        n => n as f32 * ROW_PITCH,
    }
}

/// The description container: `Width (-20, 1)`, `Height 60` (num + num2),
/// `HAlign 0.5, VAlign 1`, `PaddingBottom 12` (`:120-129`); the highlight
/// inside is `Height 56, Top 2` (`AddDescriptionPanel`, `:136-144`).
pub fn description_rect(panel: &[f32; 4]) -> [f32; 4] {
    let w = panel[2] - 20.0;
    [
        panel[0] + (panel[2] - w) * 0.5,
        panel[1] + panel[3] - 60.0 + 2.0,
        w,
        56.0,
    ]
}

/// The Apply panel: the Advanced tab's twin (`:363-372`).
pub fn apply_rect(root: &[f32; 4]) -> [f32; 4] {
    super::world_creation_advanced::apply_rect(root)
}

// --- Hits --------------------------------------------------------------------

/// What a press does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Click {
    /// `ClickSecretSeed` (`:219-236`) — the row index in [`interface_rows`].
    Row(usize),
    /// `Click_GoBack` → `GoBack` (`:381-391`) — the Advanced tab returns.
    Apply,
    None,
}

/// The click table over the live geometry.
pub fn click_at(screen: Vec2, mouse: Vec2, rows: usize, scroll: f32, attached: bool) -> Click {
    let root = root_rect(screen);
    if in_rect(mouse, apply_rect(&root)) {
        return Click::Apply;
    }
    let panel = panel_rect(&root);
    let inner = list_container_inner(&panel);
    let list = list_rect(&inner, attached);
    if in_rect(mouse, list) {
        let idx = ((mouse.y - list[1] + scroll) / ROW_PITCH) as usize;
        if idx < rows {
            let rect = row_rect(&list, idx);
            let shifted = [rect[0], rect[1] - scroll, rect[2], rect[3]];
            if in_rect(mouse, shifted) {
                return Click::Row(idx);
            }
        }
    }
    Click::None
}

/// The wheel target: the list rect or the attached scrollbar.
pub fn wheel_target(screen: Vec2, mouse: Vec2, rows: usize) -> bool {
    let root = root_rect(screen);
    let panel = panel_rect(&root);
    let inner = list_container_inner(&panel);
    let attached = content_height(rows) > inner[3];
    let list = list_rect(&inner, attached);
    in_rect(mouse, list) || (attached && in_rect(mouse, scrollbar_rect(&inner)))
}

/// `ClickSecretSeed` (`:219-236`): an enabled row disables (the text
/// leaves the entered list), a disabled row enables (the text joins it).
/// The draft's `AddSeedFromSeedmenu`/`RemoveSeedFromSeedMenu` are the
/// C#'s `_creationState2` calls (`:227`/`:233`).
pub fn toggle_row(draft: &mut NewWorldDraft, row: &SeedRow) {
    if row.enabled {
        draft.remove_seed_from_seed_menu(&row.unlock_text);
    } else {
        draft.add_seed_from_seedmenu(&row.unlock_text);
        // A parked text re-enables through the same list (the C#'s
        // `SecretSeed.Enable` flips the shared registry object).
        if let Some(i) = draft
            .disabled_secret_texts
            .iter()
            .position(|t| t == &row.unlock_text)
        {
            draft.disabled_secret_texts.remove(i);
        }
    }
}

// --- Draw --------------------------------------------------------------------

/// The panel background — `new Color(33, 43, 79) * 0.8f` (`:68`).
const PANEL_BG: [u8; 4] = [26, 34, 63, 204];
/// The description highlight — `Color.LightGray * 0.7f` (`:146`).
const DESC_HIGHLIGHT: [u8; 4] = [147, 147, 147, 178];

/// `UIWorldCreationAdvancedSecretSeedsList.Draw` (`:431-436`, minus the
/// gamepad points and the particle systems).
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    draft: &NewWorldDraft,
    screen: Vec2,
    scroll: f32,
    mouse: Vec2,
) {
    let root = root_rect(screen);
    let panel = panel_rect(&root);
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, panel, color::tint(PANEL_BG));
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );

    let rows = interface_rows(draft);
    let inner = list_container_inner(&panel);
    let content = content_height(rows.len());
    let attached = content > inner[3];
    let list = list_rect(&inner, attached);

    r.push_clip(list);
    let first = (scroll / ROW_PITCH).floor().max(0.0) as usize;
    let count = ((list[3] + LIST_PADDING) / ROW_PITCH).floor().max(1.0) as usize;
    let mut hovered_desc: Option<&SeedRow> = None;
    for slot in 0..=count {
        let Some(row) = rows.get(first + slot) else { break };
        let rect = row_rect(&list, first + slot);
        let shifted = [rect[0], rect[1] - scroll, rect[2], rect[3]];
        let hovered = in_rect(mouse, shifted);
        if hovered {
            hovered_desc = Some(row);
        }
        draw_row(r, lookups, shifted, row, hovered);
    }
    r.pop_clip();

    if attached {
        panel::draw_scrollbar(r, lookups, scrollbar_rect(&inner), inner[3], content, scroll, false);
    }

    // The description panel (`AddDescriptionPanel`, `:133-162`): the
    // highlight slice + the wrapped hovered description at 0.7
    // (PaddingLeft/Right 20, PaddingTop 4, Top 2).
    let desc = description_rect(&panel);
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_highlight,
        desc,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(DESC_HIGHLIGHT),
    );
    let body = match hovered_desc {
        Some(row) => lang::text(&description_key(row.id)),
        None => lang::text("UI.WorldDescriptionDefault"),
    };
    let mut y = desc[1] + 2.0 + 4.0;
    for line in wrap_lines(r, lookups, &body, 0.7, desc[2] - 40.0) {
        text::hud_text(r, lookups, desc[0] + 20.0, y, 0.7, [1.0; 4], &line);
        y += text::line_height(lookups) * 0.7;
    }

    // The Apply panel (`UI.Apply`, 0.65 large).
    let apply = apply_rect(&root);
    let label = lang::text("UI.Apply");
    let style = uistate::TextPanelStyle {
        text_scale: 0.65,
        large: true,
        padding: 12.0,
        bg: uistate::TEXT_PANEL_BG,
        border: uistate::TEXT_PANEL_BORDER,
    };
    let w = text::hud_text_width(r, lookups, &label, style.text_scale);
    uistate::draw_text_panel(r, lookups, apply, &style, &label, w, in_rect(mouse, apply));
}

/// One 40 px seed row (`GroupOptionButton<SecretSeed>`, `:103-118`): the
/// `PanelGrayscale` fill, the selected rim (the default
/// `InnerHighlightRim = 7`), the hovered border, the unlock text at
/// `Left 10` (`SetTextWithoutLocalization`, `:112` — `TextOriginX = 0`,
/// V-centred), and the enabled row's `IconRandomSeed` glow at the
/// top-right + (-22, 22) (`DrawGlowRing`, `:189-201` — rotation 0,
/// scale 1, centre origin).
fn draw_row(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: [f32; 4],
    row: &SeedRow,
    hovered: bool,
) {
    let (fill, accent) = super::world_creation::option_button_colors(
        row.enabled,
        hovered,
        super::world_creation::OPTION_BUTTON_COLOR,
    );
    uistate::draw_spliced_panel(
        r,
        lookups,
        lookups.cc_panel_gray,
        rect,
        10.0,
        10.0,
        10.0,
        10.0,
        color::tint(fill),
    );
    if row.enabled {
        let rim = 7.0; // the ctor default (`GroupOptionButton.cs:32`)
        uistate::draw_spliced_panel(
            r,
            lookups,
            lookups.cc_panel_highlight,
            [rect[0] + rim, rect[1] + rim, rect[2] - rim * 2.0, rect[3] - rim * 2.0],
            10.0,
            10.0,
            10.0,
            10.0,
            color::tint(accent),
        );
    }
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
            [1.0; 4],
        );
    }
    // The unlock text: `UIText` scale 1, VAlign 0.5, Left 10,
    // TextOriginX 0 — the small-font draw lifts 2 (`UIText.cs:138`).
    let ty = rect[1] + (rect[3] - text::line_height(lookups)) * 0.5 - 2.0;
    text::hud_text(r, lookups, rect[0] + 10.0, ty, 1.0, [1.0; 4], &row.unlock_text);
    // The glow ring (`DrawGlowRing`): centre at top-right + (-22, 22).
    if row.enabled {
        if let Some(sheet) = lookups.wc_icons[1] {
            let cx = rect[0] + rect[2] - 22.0;
            let cy = rect[1] + 22.0;
            let w = sheet.width as f32;
            let h = sheet.height as f32;
            super::push_whole(r, lookups, &sheet, [cx - w * 0.5, cy - h * 0.5, w, h], [1.0; 4]);
        }
    }
}

/// The wrapped description body — the bestiary's greedy word wrap.
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
        let root = root_rect(screen);
        assert_eq!(root, [550.0, 202.0, 500.0, 400.0]);
        let panel = panel_rect(&root);
        assert_eq!(panel, [550.0, 202.0, 500.0, 298.0]);
        // The container outer: Left 2, Top 4, W 480, H 238, HAlign .5 →
        // x = 550 + 2 + (500-480)*0.5 = 562; inner drops 8/12.
        let inner = list_container_inner(&panel);
        assert_eq!(inner, [562.0, 214.0, 480.0, 218.0]);
        assert_eq!(list_rect(&inner, false), [562.0, 214.0, 480.0, 218.0]);
        assert_eq!(list_rect(&inner, true), [562.0, 214.0, 455.0, 218.0]);
        assert_eq!(row_rect(&list_rect(&inner, false), 1), [562.0, 259.0, 480.0, 40.0]);
        // The description: VAlign 1 of 298, H 60 → y 440; +2 → 442, h 56.
        assert_eq!(description_rect(&panel), [560.0, 442.0, 480.0, 56.0]);
        assert_eq!(apply_rect(&root), [680.0, 509.0, 240.0, 50.0]);
    }

    /// The interface rows derive from the draft's lists, sorted by the
    /// unlock text (`CustomSort`, `:205-217`).
    #[test]
    fn the_rows_sort_by_unlock_text() {
        let mut draft = NewWorldDraft::new(0, 0);
        draft.add_seed_from_seedmenu("pumpkin season");
        draft.add_seed_from_seedmenu("ARACHNOPHOBIA");
        let rows = interface_rows(&draft);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].unlock_text, "ARACHNOPHOBIA");
        assert_eq!(rows[1].unlock_text, "pumpkin season");
        assert_eq!(rows[0].id, "noSpiderCaves");
        assert_eq!(rows[1].id, "halloweenGen");
        assert!(rows.iter().all(|r| r.enabled));
        // A parked text shows disabled.
        draft.enable_secret_seed_options(false);
        let rows = interface_rows(&draft);
        assert!(rows.iter().all(|r| !r.enabled));
    }

    /// `ClickSecretSeed` (`:219-236`): the toggle moves the text between
    /// the entered and parked lists.
    #[test]
    fn the_row_toggle_moves_the_text() {
        let mut draft = NewWorldDraft::new(0, 0);
        draft.add_seed_from_seedmenu("pumpkinseason");
        let rows = interface_rows(&draft);
        toggle_row(&mut draft, &rows[0]);
        assert!(!draft.has_entered_special_seed());
        let rows = interface_rows(&draft);
        // The removed text leaves the interface (the port's tracker is
        // the draft's lists — the C#'s settings tracker keeps it).
        assert!(rows.is_empty());
        // Re-enable through a fresh row.
        let row = SeedRow { id: "halloweenGen", unlock_text: "pumpkinseason".into(), enabled: false };
        toggle_row(&mut draft, &row);
        assert!(draft.has_entered_special_seed());
        assert_eq!(draft.secret_texts, ["pumpkinseason"]);
    }

    /// The click table: rows through the 40+5 lattice, Apply below.
    #[test]
    fn the_click_table_routes_rows_and_apply() {
        let screen = Vec2::new(1600.0, 900.0);
        // Row 0: y 214..254; row 1: 259..299.
        assert_eq!(click_at(screen, Vec2::new(700.0, 230.0), 2, 0.0, false), Click::Row(0));
        assert_eq!(click_at(screen, Vec2::new(700.0, 270.0), 2, 0.0, false), Click::Row(1));
        assert_eq!(click_at(screen, Vec2::new(700.0, 270.0), 1, 0.0, false), Click::None);
        assert_eq!(click_at(screen, Vec2::new(800.0, 520.0), 2, 0.0, false), Click::Apply);
        // Scrolled 45: row 1 lands at 214..254.
        assert_eq!(click_at(screen, Vec2::new(700.0, 230.0), 2, 45.0, false), Click::Row(1));
    }

    /// The content height and the attach rule.
    #[test]
    fn the_content_lattice_and_attach() {
        assert_eq!(content_height(0), 0.0);
        assert_eq!(content_height(1), 40.0);
        assert_eq!(content_height(6), 270.0);
        // 6 rows (270) overflow the 218 inner → the bar attaches.
        assert!(content_height(6) > 218.0);
        assert!(content_height(4) < 218.0);
    }
}
