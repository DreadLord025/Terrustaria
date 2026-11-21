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

use tr_core::bestiary::{KillTracker, UnlockState};

use super::push_whole;
use super::text;
use super::uistate::{self, Style};
use crate::assets::{self, Lookups};
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

/// The grid area — the `-12px wide, -4px tall` bestiary space above the
/// bottom bar (`:139-146`).
fn grid_rect(panel: &[f32; 4]) -> [f32; 4] {
    [
        panel[0] + 6.0,
        panel[1] + 40.0,
        panel[2] - 12.0,
        panel[3] - 40.0 - 4.0,
    ]
}

/// `UIBestiaryEntryGrid.GetEntriesToShow` (`:95-100`): the cells per
/// row/column are the space divided by 72.
fn grid_metrics(rect: &[f32; 4]) -> (usize, usize) {
    let cols = ((rect[2] / CELL) as usize).max(1);
    let rows = ((rect[3] / CELL) as usize).max(1);
    // The rows grow with the entry count — the vanilla's bestiary scrolls
    // (`UIBestiaryGrid`'s scroll bars land with the UI-scroll work); the
    // fixed rows clipped the roster at the town-NPC growth.
    let need = (crate::bestiary::ENTRIES.len() + cols - 1) / cols;
    (cols, rows.max(need))
}

/// The entry index under the cursor (the grid's per-cell centre placement
/// simplifies to a fixed lattice — the cards are `HAlign/VAlign = 0.5` with
/// the fractional `Left/Top` offsets `:104-108` produce).
pub fn card_at(screen: Vec2, mouse: Vec2) -> Option<usize> {
    let panel = panel_rect(screen);
    let rect = grid_rect(&panel);
    let (cols, rows) = grid_metrics(&rect);
    // `GetEntriesToShow`'s raw CELL lattice (the space divided by 72 — no
    // insets): the draw and the hit-test share it, so the overflow rows
    // keep the hits.
    let cx = ((mouse.x - rect[0]) / CELL) as usize;
    let cy = ((mouse.y - rect[1]) / CELL) as usize;
    if cx >= cols || cy >= rows {
        return None;
    }
    let idx = cy * cols + cx;
    if idx < crate::bestiary::ENTRIES.len() {
        Some(idx)
    } else {
        None
    }
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

/// The completion bar's rect — `FillProgressBottomBar`
/// (`UIBestiaryTest.cs:158-179`): the `UIColoredSliderSimple` fills the
/// bottom strip (`Width (0, 1)`, `Height 15`, `VAlign 1`, `:168-171`); the
/// strip itself is the panel's bottom 20 px row (`BuildPage`, `:123-131`)
/// inset by `UIPanel`'s default 12 px side padding.
pub fn progress_bar_rect(panel: &[f32; 4]) -> [f32; 4] {
    [
        panel[0] + 12.0,
        panel[1] + panel[3] - 12.0 - 15.0,
        panel[2] - 24.0,
        15.0,
    ]
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

/// The whole screen: the dim backdrop, the container panel, the title, the
/// grid of entry cards (or the selected entry's info page), the completion
/// bar and the Back panel.
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &KillTracker,
    selected: Option<usize>,
    mouse: Vec2,
) {
    // The state's background — the game keeps rendering behind
    // (`IngameFancyUI`), dimmed the way the port's other overlays are.
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let _ = (aw, ah);
    r.push_ui_sprite([0.0, 0.0], [screen.x, screen.y], r.white_uv(), BACKDROP);
    draw_panel_chrome(r, lookups, screen);

    match selected {
        Some(idx) => draw_info_page(r, lookups, screen, tracker, idx, mouse),
        None => {
            draw_grid(r, lookups, screen, tracker, mouse);
            // `_indexesRangeText` (`UIBestiaryTest.cs:248-254` +
            // `UpdateBestiaryGridRange`, `:345-348`): the "1-N (N)" range
            // panel in the top strip. The port's grid grows its rows to
            // fit the roster (no paging yet — the Back/Forward page
            // buttons defer with the filters/sort), so the range always
            // starts at entry 1.
            let panel = panel_rect(screen);
            let rect = grid_rect(&panel);
            let (cols, rows) = grid_metrics(&rect);
            let range = grid_range_text(0, cols * rows, crate::bestiary::ENTRIES.len());
            let w = text::hud_text_width(r, lookups, &range, 0.8);
            text::hud_text(
                r,
                lookups,
                panel[0] + panel[2] - 12.0 - w,
                panel[1] + 14.0,
                0.8,
                [1.0; 4],
                &range,
            );
        }
    }

    // The completion bar — `FillProgressBottomBar` (`UIBestiaryTest.cs:158-179`)
    // + `ShowStats_Completion`/`GetCompletionPercentText` (`:181-196`):
    // `GetUnlockProgress` (`:497-513`) counts every entry past
    // `NotKnownAtAll_0` — NOT only the fully-unlocked ones (the old tier-4
    // count read far too low); `CompletionPercent`
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
    let text_w = text::hud_text_width(r, lookups, "Back", style.text_scale);
    super::uistate::draw_text_panel(
        r,
        lookups,
        r0,
        &style,
        "Back",
        text_w,
        back_hit(screen, mouse),
    );
}

fn draw_panel_chrome(r: &mut SpriteRenderer, lookups: &Lookups, screen: Vec2) {
    // The panel body: `UIPanel`'s `BackgroundColor = (33, 43, 79) * 0.8`
    // (`UIBestiaryTest.cs:84-88`), drawn as a flat quad (the port's panel
    // style for the full-screen states).
    let p = panel_rect(screen);
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
    // The header text — the screen has no title element; the port labels the
    // panel with the inventory button's tooltip name.
    text::hud_text(
        r,
        lookups,
        p[0] + 12.0,
        p[1] + 10.0,
        1.0,
        [1.0; 4],
        "Bestiary",
    );
}

fn draw_grid(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &KillTracker,
    mouse: Vec2,
) {
    let panel = panel_rect(screen);
    let rect = grid_rect(&panel);
    let (cols, rows) = grid_metrics(&rect);
    let capacity = cols * rows;
    // `FillBestiarySpaceWithEntries`' page start (`UIBestiaryEntryGrid.cs:37-71`):
    // `FixBestiaryRange(0, capacity)` clamps `_atEntryIndex` before the fill.
    // The port grows its rows to fit the whole roster (no Back/Forward paging
    // yet — deferred with the filters/sort), so the offset is 0 and the clamp
    // pins the start to 0; the math is the C#'s so paging lands on it later.
    let at = fix_grid_range(0, 0, capacity, crate::bestiary::ENTRIES.len());
    for (slot, entry) in crate::bestiary::ENTRIES
        .iter()
        .skip(at)
        .take(capacity)
        .enumerate()
    {
        let idx = at + slot;
        let col = slot % cols;
        let row = slot / cols;
        let x = rect[0] + col as f32 * CELL;
        let y = rect[1] + row as f32 * CELL;
        let cell = [x, y, CELL, CELL];
        let hover = card_at(screen, mouse) == Some(idx);
        draw_card(r, lookups, cell, entry, tracker, idx, hover);
    }
    // `UIBestiaryEntryButton.DrawSelf` (`UIBestiaryEntryButton.cs:142-148`):
    // the hovered card prints the icon's hover text — the NPC name once
    // known, "???" while locked (`UnlockableNPCEntryIcon.GetHoverText`,
    // `:209-221`).
    if let Some(idx) = card_at(screen, mouse) {
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

/// One `UIBestiaryEntryButton` (`:20-86`): the inner 68 px box holds
/// `Slot_Back` + the portrait (or the 15 % white `Icon_Locked`), the
/// `Slot_Overlay` at 60 % white and `Slot_Front` stack over it, the
/// `Slot_Selection` frame appears on hover and the display index prints at
/// (10, 10) scale 0.9.
fn draw_card(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    cell: [f32; 4],
    entry: &crate::bestiary::BestiaryEntryDef,
    tracker: &KillTracker,
    idx: usize,
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
        // `UIBestiaryEntryIcon.DrawSelf`'s locked branch (`:44-56`): the
        // texture at white * 0.15, centred.
        push_whole(r, lookups, &locked, inner, [0.15; 4]);
    }
    r.pop_clip();
    if let Some(overlay) = lookups.ui_bestiary_slot_overlay {
        push_whole(r, lookups, &overlay, inner, [0.6; 4]);
    }
    if let Some(front) = lookups.ui_bestiary_slot_front {
        push_whole(r, lookups, &front, inner, [1.0; 4]);
    }
    if hover {
        if let Some(sel) = lookups.ui_bestiary_slot_selection {
            push_whole(r, lookups, &sel, cell, [1.0; 4]);
        }
    }
    // The display index (`:58-66` — `IBestiaryEntryDisplayIndex`'s number).
    text::hud_text(
        r,
        lookups,
        cell[0] + 10.0,
        cell[1] + 10.0,
        0.9,
        [1.0; 4],
        &format!("{}", idx + 1),
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
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::DD2LightningBug, _) => (sheet_i, 24.0, 30.0, 24.0, white),
        (NpcKind::WalkingAntlion, _) => (sheet_i, 26.0, 26.0, 26.0, white),
        (NpcKind::FlyingAntlion, _) => (sheet_i, 22.0, 30.0, 22.0, white),
        (NpcKind::LarvaeAntlion, _) => (sheet_i, 18.0, 18.0, 18.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::FairyCritterPink, _) => (sheet_i, 20.0, 18.0, 20.0, white),
        (NpcKind::FairyCritterGreen, _) => (sheet_i, 20.0, 18.0, 20.0, white),
        (NpcKind::FairyCritterBlue, _) => (sheet_i, 20.0, 18.0, 20.0, white),
        (NpcKind::ZombieMerman, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::EyeballFlyingFish, _) => (sheet_i, 18.0, 32.0, 18.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::TorchZombie, _) => (sheet_i, 40.0, 18.0, 40.0, white),
        (NpcKind::GoldGoldfish, _) => (sheet_i, 18.0, 20.0, 18.0, white),
        (NpcKind::WindyBalloon, _) => (sheet_i, 20.0, 20.0, 20.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::RedDragonfly, _) => (sheet_i, 10.0, 10.0, 10.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::GoldDragonfly, _) => (sheet_i, 10.0, 10.0, 10.0, white),
        (NpcKind::Seagull, _) => (sheet_i, 26.0, 22.0, 26.0, white),
        (NpcKind::LadyBug, _) => (sheet_i, 10.0, 10.0, 10.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::GoldLadyBug, _) => (sheet_i, 10.0, 10.0, 10.0, white),
        (NpcKind::Maggot, _) => (sheet_i, 4.0, 10.0, 4.0, white),
        (NpcKind::Pupfish, _) => (sheet_i, 18.0, 20.0, 18.0, white),
        (NpcKind::Grebe, _) => (sheet_i, 26.0, 22.0, 26.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::Rat, _) => (sheet_i, 12.0, 14.0, 12.0, white),
        (NpcKind::Owl, _) => (sheet_i, 34.0, 18.0, 34.0, white),
        (NpcKind::WaterStrider, _) => (sheet_i, 10.0, 10.0, 10.0, white),
        (NpcKind::GoldWaterStrider, _) => (sheet_i, 10.0, 10.0, 10.0, white),
        (NpcKind::ExplosiveBunny, _) => (sheet_i, 20.0, 18.0, 20.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::Dolphin, _) => (sheet_i, 18.0, 20.0, 18.0, white),
        (NpcKind::Turtle, _) => (sheet_i, 20.0, 22.0, 20.0, white),
        (NpcKind::TurtleJungle, _) => (sheet_i, 20.0, 22.0, 20.0, white),
        (NpcKind::BloodNautilus, _) => (sheet_i, 100.0, 100.0, 100.0, white),
        (NpcKind::BloodSquid, _) => (sheet_i, 44.0, 44.0, 44.0, white),
        // W4-d42 - the roster's portraits rest the kinds' SetDefaults
        // boxes on their own sheets (the WyvernHead literal-size
        // precedent - the sheets are deferred, the index folds to 0).
        (NpcKind::GoblinShark, _) => (sheet_i, 58.0, 34.0, 58.0, white),
        (NpcKind::BloodEelHead, _) => (sheet_i, 28.0, 28.0, 28.0, white),
        (NpcKind::Gnome, _) => (sheet_i, 30.0, 14.0, 30.0, white),
    }
}

/// `UIBestiaryEntryInfoPage`'s content for the selected entry: the
/// `Portrait_Front` backdrop, the big portrait, the name (tier >= 1), the
/// stat rows (tier >= 2 — `Stat_HP`/`Stat_Attack`/`Stat_Defense`/
/// `Stat_Knockback`/`Stat_Copper`), the kill progress and the drop lines
/// (rates only from tier 4).
fn draw_info_page(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    tracker: &KillTracker,
    idx: usize,
    mouse: Vec2,
) {
    let panel = panel_rect(screen);
    let entry = &crate::bestiary::ENTRIES[idx];
    let state = crate::bestiary::unlock_state(entry, tracker);
    if let Some(portrait_bg) = lookups.ui_bestiary_portrait {
        // The portrait card left of the info (the pretty-portrait mode's
        // 72 px card scaled up).
        let size = 216.0;
        let px = panel[0] + 24.0;
        let py = panel[1] + 60.0;
        let inner = [px + 2.0, py + 2.0, size - 4.0, size - 4.0];
        // The portrait card's inner container is `OverflowHidden` too
        // (`UIBestiaryNPCEntryPortrait.cs:20-28`), and its icon draws with
        // `isPortrait: true` (`:56`) — the `PortraitScale` /
        // `PortraitPositionX-YOverride` arms of the offset table.
        r.push_clip(inner);
        if let Some(back) = lookups.ui_bestiary_slot_back {
            push_whole(r, lookups, &back, inner, [1.0; 4]);
        }
        if state.known() {
            draw_portrait(r, lookups, entry, inner, true);
        } else if let Some(locked) = lookups.ui_bestiary_locked {
            push_whole(r, lookups, &locked, inner, [0.15; 4]);
        }
        r.pop_clip();
        if let Some(front) = lookups.ui_bestiary_slot_front {
            push_whole(r, lookups, &front, inner, [1.0; 4]);
        }
    }
    let tx = panel[0] + 264.0;
    let mut ty = panel[1] + 60.0;
    // The name — tier 1+ (`GetHoverText`'s `???` gate,
    // `UnlockableNPCEntryIcon.cs:213-226`).
    // `Lang.GetNPCNameValue(netId)` (`Lang.cs:220-223`) — the localized NPC
    // name (the English column stays the fallback for a missing key).
    let shown_name = if state.known() {
        let localized = crate::lang::npc_name(entry.net_id);
        if localized.is_empty() {
            entry.name.to_string()
        } else {
            localized
        }
    } else {
        "???".to_string()
    };
    text::hud_text(r, lookups, tx, ty, 1.2, [1.0, 0.9, 0.4, 1.0], &shown_name);
    ty += 34.0;

    // The stat rows — `NPCStatsReportInfoElement.ProvideUIElement`
    // (`NPCStatsReportInfoElement.cs:57-128`): the block appears from tier 1
    // (`== NotKnownAtAll_0` returns null, `:59-62`), but every value prints
    // "???" until tier 2 (`:124-128`). The knockback column is the
    // High/Med/Low/None label, not the raw resist (`:123`).
    if state.known() {
        let stats =
            crate::npc::entry_stats(entry.kind, entry.variant.unwrap_or(SlimeVariant::Blue));
        let known = state >= UnlockState::CanShowStats;
        let masked = |v: String| if known { v } else { "???".to_string() };
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_hp,
            &masked(format!("{}", stats.hp)),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_attack,
            &masked(format!("{}", stats.damage)),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_defense,
            &masked(format!("{}", stats.defense)),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_knockback,
            &knockback_label(stats.kb, known),
        );
        stat_row(
            r,
            lookups,
            &mut ty,
            tx,
            lookups.ui_bestiary_stat_copper,
            &masked(format!("{}", stats.value as i64)),
        );
        ty += 10.0;
    }

    // The flavor text — `FlavorTextBestiaryInfoElement` (tier 2+,
    // `FlavorTextBestiaryInfoElement.cs:20-23`), the wrapped 0.8-scale row
    // in the (43, 56, 101) stat panel (`:24-33`); the panel grows with the
    // wrapped text (`AddDynamicResize`, `:47-53`).
    if let Some(flavor) = flavor_text(entry, state) {
        let panel_w = 230.0 - 11.0; // `Width (-11, 1)` of the 230 px info page (`:26`, `UIBestiaryEntryInfoPage.cs:30`)
        let lines = wrap_text(r, lookups, &flavor, 0.8, panel_w - 8.0);
        let line_h = text::line_height(lookups) * 0.8;
        let h = lines.len() as f32 * line_h + 8.0;
        r.push_ui_sprite(
            [tx, ty],
            [panel_w, h],
            r.white_uv(),
            super::color::tint([43, 56, 101, 255]),
        );
        for (i, line) in lines.iter().enumerate() {
            text::hud_text(r, lookups, tx + 4.0, ty + 4.0 + i as f32 * line_h, 0.8, [1.0; 4], line);
        }
        ty += h + 8.0;
    }

    // The kill counter — `NPCKillCounterInfoElement`
    // (`NPCKillCounterInfoElement.cs:24-96`): the raw count (no "/needed" —
    // the progress fraction is the port's own earlier stand-in) from the
    // first kill, with the `BestiaryInfo.Slain` hover (`:90-96`).
    let (kills, _needed) = crate::bestiary::kill_progress(entry, tracker);
    if let Some(count) = kill_counter_text(kills) {
        text::hud_text(r, lookups, tx, ty, 0.85, [0.8, 0.85, 1.0, 1.0], &count);
        if super::in_rect(mouse, [tx, ty, 60.0, 20.0]) {
            super::tooltip::draw_mouse_text(
                r,
                lookups,
                mouse,
                &crate::lang::text("BestiaryInfo.Slain"),
                screen,
                super::UiPulse::default(),
            );
        }
        ty += 26.0;
    }

    // The drops — `ItemDropBestiaryInfoElement` shows its lines from tier 2
    // (`ItemDropBestiaryInfoElement.cs:20-23`); below tier 3 each line is
    // the locked icon (`UIBestiaryInfoItemLine.cs:36-55`), tier 3 prints
    // "???" for the rate, tier 4 the stack range + rate (`:92-109`).
    if state >= UnlockState::CanShowStats {
        text::hud_text(r, lookups, tx, ty, 0.8, [1.0; 4], "Drops:");
        ty += 22.0;
        let rows = crate::loot_db::drop_rows(entry.net_id);
        for row in &rows {
            let name = crate::registry::name(row.item_id as u16);
            let rate = drop_rate_text(state, row.drop_rate, row.stack_min, row.stack_max);
            let line = if state >= UnlockState::CanShowDropsWithoutDropRates {
                format!("  {name} {rate}")
            } else {
                // The locked-icon line (`UIBestiaryInfoItemLine.cs:39-54`):
                // `Icon_Locked` at 0.55 scale, centred in the 32 px row.
                if let Some(locked) = lookups.ui_bestiary_locked {
                    let lw = locked.width as f32 * 0.55;
                    let lh = locked.height as f32 * 0.55;
                    push_whole(
                        r,
                        lookups,
                        &locked,
                        [tx + (230.0 - lw) * 0.5, ty + (32.0 - lh) * 0.5, lw, lh],
                        [1.0; 4],
                    );
                }
                ty += 32.0;
                continue;
            };
            text::hud_text(r, lookups, tx, ty, 0.8, [1.0; 4], &line);
            ty += 22.0;
        }
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

/// One `Stat_*` icon + value row (the info page's stat list style).
fn stat_row(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    ty: &mut f32,
    tx: f32,
    sheet: Option<tr_render::SheetRect>,
    value: &str,
) {
    if let Some(sheet) = sheet {
        push_whole(r, lookups, &sheet, [tx, *ty, 20.0, 20.0], [1.0; 4]);
    }
    text::hud_text(r, lookups, tx + 28.0, *ty + 2.0, 0.8, [1.0; 4], value);
    *ty += 24.0;
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
        // A 1280x720 screen: the panel is 900 wide (0.875*1280 = 1120 capped)
        // and the grid fills rows of 72 px cells; every entry stays reachable.
        let screen = Vec2::new(1280.0, 720.0);
        let panel = panel_rect(screen);
        assert_eq!(panel[2], 900.0);
        let rect = grid_rect(&panel);
        let (cols, rows) = grid_metrics(&rect);
        assert!(cols * rows >= crate::bestiary::ENTRIES.len());
        // The first cell's centre hits entry 0 — the probe rides
        // `card_at`'s own lattice (the raw CELL grid, no insets) so the
        // overflow rows keep the hit.
        let first_x = rect[0] + CELL * 0.5;
        let first_y = rect[1] + CELL * 0.5;
        assert_eq!(card_at(screen, Vec2::new(first_x, first_y)), Some(0));
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
        // VAlign 1 in the 20 px bottom strip, the panel's 12 px side inset.
        let panel = panel_rect(Vec2::new(1600.0, 900.0));
        let bar = progress_bar_rect(&panel);
        assert_eq!(bar[3], 15.0);
        assert_eq!(bar[0], panel[0] + 12.0);
        assert_eq!(bar[2], panel[2] - 24.0);
        // Bottom-aligned inside the panel (the strip's bottom 12 px pad).
        assert_eq!(bar[1] + bar[3], panel[1] + panel[3] - 12.0);
    }

    #[test]
    fn completion_counts_every_known_entry_not_only_tier_4() {
        // `GetUnlockProgress` (`UIBestiaryTest.cs:497-513`) counts
        // `UnlockState > NotKnownAtAll_0` — one kill of the first entry
        // already moves the bar (the old tier-4-only count did not).
        let mut kills = KillTracker::new();
        let first = &crate::bestiary::ENTRIES[0];
        assert_eq!(crate::bestiary::completion_percent(&kills), 0.0);
        kills.register_kill(first.credit_id);
        let pct = crate::bestiary::completion_percent(&kills);
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
