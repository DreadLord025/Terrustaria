//! The debug loadouts list (`UIDebugLoadoutsList`,
//! `GameContent/UI/States/UIDebugLoadoutsList.cs:14-146`) and its row element
//! (`UIDebugLoadoutItem`, `Elements/UIDebugLoadoutItem.cs:12-149`) — W4-d85
//! row 8 (+ the row-9 element half). The commands list's twin page: the same
//! container/panel/search geometry, the "Loadouts" title, the 70 px rows
//! with the four `UIItemIcon` groups, and the static `_lastSearch` the page
//! re-seeds its search bar with (`:20`, `:85`).
//!
//! Opened by `/loadouts` (`ToolkitDebugCommands.cs:884-889`). A row click
//! closes the overlay, applies the loadout to the player and appends the
//! yellow "Loadout {name} set!" notice (`UIDebugLoadoutItem.LeftClick`,
//! `:143-148`).
//!
//! Deviations (docs/ANALYSIS.md "Debug lists (W4-d85)"): the hover tooltip
//! (`MouseOver`'s hardcoded "test" wrap, `:132-133`) is not drawn; the
//! minion respawn half of `Apply` is deferred (see `loadout.rs`).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::text::{self, BorderFont};
use super::uistate;
use super::{color, in_rect, panel};
use crate::assets::Lookups;
use crate::item::ItemSlot;
use crate::lang;
use crate::loadout::ScenarioTestLoadout;

/// One `UIDebugLoadoutItem` — the parsed config plus the friendly name
/// (`new FileInfo(path).Name.Replace(".json", "")`, `UIDebugLoadoutsList.
/// cs:140`).
#[derive(Clone, Debug, PartialEq)]
pub struct LoadoutEntry {
    pub name: String,
    pub loadout: ScenarioTestLoadout,
}

/// `PopulateLoadouts` (`UIDebugLoadoutsList.cs:134-145`): every `*json`
/// file under `Main.SavePath/Loadouts` that parses; unreadable files are
/// skipped (`TryReadFromFilePath`'s catch).
pub fn populate(dir: &std::path::Path) -> Vec<LoadoutEntry> {
    let mut out = Vec::new();
    for path in crate::loadout::find_all(dir) {
        let Some(loadout) = crate::loadout::try_read_from_file_path(&path) else {
            continue;
        };
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .replace(".json", "");
        out.push(LoadoutEntry { name, loadout });
    }
    out
}

/// `UIDebugLoadoutItem`'s row height — `Height.Set(70f, 0f)` (`:35`).
pub const ROW_H: f32 = 70.0;
/// `ListPadding = 5f` (`UIDebugLoadoutsList.cs:60`).
pub const LIST_PADDING: f32 = 5.0;
/// The row stride (`UIList.RecalculateChildren`, `UIList.cs:119-127`).
pub const ROW_PITCH: f32 = ROW_H + LIST_PADDING;

/// `DoesCommandMatchSearch` (`UIDebugLoadoutsList.cs:107-114`): the name
/// only (the loadouts twin has no description/help fields).
pub fn loadout_matches(lower_contents: &str, entry: &LoadoutEntry) -> bool {
    entry.name.to_lowercase().contains(lower_contents)
}

/// `searchbar_OnSearchContentsChanged` (`:88-105`): the same null → "" →
/// lower+trim → whitespace-shows-all shape as the commands list.
pub fn visible_loadouts(entries: &[LoadoutEntry], search: &str) -> Vec<usize> {
    let text = search.to_lowercase();
    let text = text.trim();
    let all = text.is_empty();
    entries
        .iter()
        .enumerate()
        .filter(|(_, e)| all || loadout_matches(text, e))
        .map(|(i, _)| i)
        .collect()
}

// --- Geometry (`BuildPage`, `UIDebugLoadoutsList.cs:31-86`) -------------------
//
// The container/panel/search/list/scrollbar/title/back chain is the commands
// list's byte-for-byte (`:34-83` vs `UIDebugCommandsList.cs:33-82`); the
// shared rects live on `debug_commands` and are re-exported here so each
// screen keeps its own module surface.

pub use super::debug_commands::{
    back_rect, container_rect, list_rect, panel_rect, scrollbar_rect, search_rect, title_rect,
};

/// The row rect at `index` — the 70 px lattice.
pub fn row_rect(list: &[f32; 4], index: usize) -> [f32; 4] {
    [list[0], list[1] + index as f32 * ROW_PITCH, list[2], ROW_H]
}

/// The list content height (`UIList.cs:119-127`): N*(70+5); one row 70.
pub fn content_height(rows: usize) -> f32 {
    match rows {
        0 => 0.0,
        1 => ROW_H,
        n => n as f32 * ROW_PITCH,
    }
}

/// The four `PositionItems` groups (`UIDebugLoadoutItem.cs:88-91`):
/// `(horizontalStart, horizontalEnd, vertical)` per group — hotbar
/// (0..5), ammo (54..58), armor head/body/legs (0..3), accessories (3..10).
pub const ICON_GROUPS: [(f32, f32, f32); 4] = [
    (0.1, 0.3, 0.75),
    (0.35, 0.45, 0.75),
    (0.65, 0.75, 0.25),
    (0.6, 0.9, 0.75),
];

/// The item arrays a row previews (`UIDebugLoadoutItem` ctor, `:47-87`):
/// the inventory with `Items_Inventory` then `Items_Ammo` applied, the
/// armor with `Items_Armor`. Returned as the four icon groups' resolved
/// `ItemSlot`s (a `None` cell draws no icon, the C#'s null check `:56`).
pub fn preview_groups(loadout: &ScenarioTestLoadout) -> [Vec<ItemSlot>; 4] {
    let mut inventory = vec![None; 58];
    let mut armor = vec![None; 20];
    crate::loadout::apply_items_sparse(&mut inventory, &loadout.items_inventory, 0);
    crate::loadout::apply_items_sparse(&mut inventory, &loadout.items_ammo, 0);
    crate::loadout::apply_items_sparse(&mut armor, &loadout.items_armor, 0);
    let group = |slots: &[Option<ItemSlot>], range: std::ops::Range<usize>| -> Vec<ItemSlot> {
        range.filter_map(|i| slots.get(i).copied().flatten()).collect()
    };
    [
        group(&inventory, 0..5),
        group(&inventory, 54..58),
        group(&armor, 0..3),
        group(&armor, 3..10),
    ]
}

/// One icon's centre in a group (`PositionItems`, `UIDebugLoadoutItem.cs:
/// 94-112`): the group element spans `[start, end]` of the row width at
/// `vertical` of its height; icon i sits at `HAlign = i/(n-1)` (0.5 when
/// alone), each icon 32x32 (`UIItemIcon.cs:16-17`).
pub fn icon_center(row: &[f32; 4], group: usize, i: usize, n: usize) -> (f32, f32) {
    let (start, end, vertical) = ICON_GROUPS[group];
    let gx = row[0] + start * row[2];
    let gw = (end - start) * row[2];
    let h_align = if n > 1 { i as f32 / (n - 1) as f32 } else { 0.5 };
    // The icon element's own 32 px width rides the alignment shift
    // (`UIElement.cs:319`).
    let x = gx + gw * h_align - 32.0 * h_align + 16.0;
    // `Top = vertical%` of the row; the 32 px icon has no VAlign, so its
    // centre is top + 16.
    let y = row[1] + vertical * row[3] + 16.0;
    (x, y)
}

// --- Hits --------------------------------------------------------------------

/// What a press does.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Click {
    /// `GoBackClick` → `IngameFancyUI.Close()` (`:116-119`).
    Back,
    /// The search box (`Click_SearchArea`).
    Search,
    /// The cancel cross (`searchCancelButton_OnClick`).
    SearchCancel,
    /// A loadout row (`UIDebugLoadoutItem.LeftClick`, `:143-148`) — the
    /// index into the entries vector.
    Row(usize),
    None,
}

/// The click table — the commands list's shape over the 70 px lattice.
pub fn click_at(screen: Vec2, mouse: Vec2, scroll: f32, rows: &[usize]) -> Click {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    if in_rect(mouse, back_rect(&container)) {
        return Click::Back;
    }
    let search = search_rect(&panel);
    if in_rect(mouse, search) {
        let box_x = search[0] + 25.0;
        let box_w = search[2] - 25.0;
        let cancel = [box_x + box_w - 22.0, search[1], 20.0, search[3]];
        if in_rect(mouse, cancel) {
            return Click::SearchCancel;
        }
        return Click::Search;
    }
    let list = list_rect(&panel);
    if in_rect(mouse, list) {
        let idx = ((mouse.y - list[1] + scroll) / ROW_PITCH) as usize;
        if let Some(&row) = rows.get(idx) {
            let rect = row_rect(&list, idx);
            let shifted = [rect[0], rect[1] - scroll, rect[2], rect[3]];
            if in_rect(mouse, shifted) {
                return Click::Row(row);
            }
        }
    }
    Click::None
}

/// The wheel target (the list rect or the attached scrollbar).
pub fn wheel_target(screen: Vec2, mouse: Vec2, rows: usize) -> bool {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    let list = list_rect(&panel);
    let attaches = content_height(rows) > list[3];
    in_rect(mouse, list) || (attaches && in_rect(mouse, scrollbar_rect(&panel)))
}

// --- Draw --------------------------------------------------------------------

/// The panel background — `new Color(33, 43, 79) * 0.95f` (`:45`).
const PANEL_BG: [u8; 4] = [31, 40, 75, 242];
/// The title background — `new Color(73, 94, 171)` (`:66`).
const TITLE_BG: [u8; 4] = [73, 94, 171, 255];
/// The search box panel (`UIWrappedSearchBar.cs:117-118`).
const SEARCH_BG: [u8; 4] = [35, 40, 83, 255];
/// The row idle pair — `(76,90,149)*0.2` / `(50,60,86)*0.2`
/// (`UIDebugLoadoutItem.cs:38-39`, XNA byte truncation).
const ROW_BG: [u8; 4] = [15, 18, 29, 51];
const ROW_BORDER: [u8; 4] = [10, 12, 17, 51];
/// The row hover pair (`MouseOver`, `:129-130`).
const ROW_HOVER_BG: [u8; 4] = [76, 90, 149, 255];
const ROW_HOVER_BORDER: [u8; 4] = [50, 60, 86, 255];
/// The port's overlay dim.
const BACKDROP: [f32; 4] = [0.0, 0.0, 0.0, 0.5];

/// `UIDebugLoadoutsList.Draw` with the port's overlay treatment.
pub fn draw_screen(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    screen: Vec2,
    entries: &[LoadoutEntry],
    search: &str,
    scroll: f32,
    mouse: Vec2,
) {
    let container = container_rect(screen);
    let panel = panel_rect(&container);
    r.push_ui_sprite([0.0, 0.0], [screen.x, screen.y], r.white_uv(), BACKDROP);

    panel::draw_panel(r, lookups, lookups.ui_panel_bg, panel, color::tint(PANEL_BG));
    panel::draw_panel(
        r,
        lookups,
        lookups.ui_panel_border,
        panel,
        color::tint(uistate::TEXT_PANEL_BORDER),
    );

    draw_search_bar(r, lookups, &panel, search, mouse);
    draw_title(r, lookups, &container);
    draw_back(r, lookups, &container, mouse);

    let rows = visible_loadouts(entries, search);
    let list = list_rect(&panel);
    r.push_clip(list);
    let first = (scroll / ROW_PITCH).floor().max(0.0) as usize;
    let count = ((list[3] + LIST_PADDING) / ROW_PITCH).floor().max(1.0) as usize;
    for slot in 0..=count {
        let Some(&row) = rows.get(first + slot) else { break };
        let rect = row_rect(&list, first + slot);
        let shifted = [rect[0], rect[1] - scroll, rect[2], rect[3]];
        draw_row(r, lookups, shifted, &entries[row], in_rect(mouse, shifted));
    }
    r.pop_clip();

    let content = content_height(rows.len());
    if content > list[3] {
        panel::draw_scrollbar(
            r,
            lookups,
            scrollbar_rect(&panel),
            list[3],
            content,
            scroll,
            false,
        );
    }
}

/// The search bar — the commands list's copy (`UIWrappedSearchBar`'s
/// box/magnifier/cancel trio).
fn draw_search_bar(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    panel: &[f32; 4],
    search: &str,
    mouse: Vec2,
) {
    let rect = search_rect(panel);
    let box_rect = [rect[0] + 25.0, rect[1], rect[2] - 25.0, rect[3]];
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, box_rect, color::tint(SEARCH_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, box_rect, color::tint(SEARCH_BG));
    let (text, tint) = if search.is_empty() {
        (lang::text("UI.PlayerNameSlot"), color::tint([128, 128, 128, 255]))
    } else {
        (search.to_string(), [1.0; 4])
    };
    text::hud_text(r, lookups, box_rect[0] + 4.0, box_rect[1] + 5.0, 0.8, tint, &text);
    if let Some(sheet) = lookups.ui_search_icon {
        super::push_whole(
            r,
            lookups,
            &sheet,
            [rect[0], rect[1] + (rect[3] - 22.0) * 0.5, 22.0, 22.0],
            [1.0; 4],
        );
    }
    if let Some(sheet) = lookups.ui_search_cancel {
        let w = sheet.width as f32;
        let h = sheet.height as f32;
        let dst = [box_rect[0] + box_rect[2] - w - 2.0, rect[1] + (rect[3] - h) * 0.5, w, h];
        super::push_whole(
            r,
            lookups,
            &sheet,
            dst,
            if in_rect(mouse, dst) { [1.0; 4] } else { [0.7, 0.7, 0.7, 1.0] },
        );
    }
}

/// The "Loadouts" title (`UITextPanel<string>("Loadouts", 1f, large)`,
/// `SetPadding(13)`, `:62-67`).
fn draw_title(r: &mut SpriteRenderer, lookups: &Lookups, container: &[f32; 4]) {
    let label = "Loadouts";
    let w = match &lookups.death_font {
        Some(f) => text::font_width(f, label, 1.0),
        None => r.text_width(label, 1.0),
    };
    let rect = title_rect(container, w);
    let style = uistate::TextPanelStyle {
        text_scale: 1.0,
        large: true,
        padding: 13.0,
        bg: TITLE_BG,
        border: uistate::TEXT_PANEL_BORDER,
    };
    uistate::draw_text_panel(r, lookups, rect, &style, label, w, false);
}

/// The Back panel (`UI.Back`, 0.7 large, the faded pair, `:68-77`).
fn draw_back(r: &mut SpriteRenderer, lookups: &Lookups, container: &[f32; 4], mouse: Vec2) {
    let rect = back_rect(container);
    let label = lang::text("UI.Back");
    let style = super::cards::fat_button_style();
    let w = text::hud_text_width(r, lookups, &label, style.text_scale);
    uistate::draw_text_panel(r, lookups, rect, &style, &label, w, in_rect(mouse, rect));
}

/// One `UIDebugLoadoutItem` (`DrawSelf`, `:114-124` + the ctor's icon
/// groups): the tinted panel, the name at inner + (12, -2) scale 1.1, and
/// the four `UIItemIcon` groups (`ItemSlot.DrawItemIcon`, ctx 31, size 32).
fn draw_row(
    r: &mut SpriteRenderer,
    lookups: &Lookups,
    rect: [f32; 4],
    entry: &LoadoutEntry,
    hover: bool,
) {
    let (bg, border) = if hover {
        (ROW_HOVER_BG, ROW_HOVER_BORDER)
    } else {
        (ROW_BG, ROW_BORDER)
    };
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, rect, color::tint(bg));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, rect, color::tint(border));
    let inner = uistate::inner(rect, [6.0; 4]);
    text::draw_border_string(
        r,
        lookups,
        BorderFont::Ui,
        inner[0] + 12.0,
        inner[1] - 2.0,
        1.1,
        &[[1.0, 1.0]],
        [1.0; 4],
        &entry.name,
    );
    // The icon groups (`PositionItems` + `UIItemIcon.DrawSelf`,
    // `UIItemIcon.cs:21-25`: `DrawItemIcon(item, 31, center, item.scale,
    // 32, White)`).
    let groups = preview_groups(&entry.loadout);
    for (g, items) in groups.iter().enumerate() {
        for (i, item) in items.iter().enumerate() {
            let center = icon_center(&rect, g, i, items.len());
            super::slot::draw_item_icon_32(r, lookups, item, center);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::loadout::{deserialize, ItemRecord};

    fn entry(name: &str) -> LoadoutEntry {
        LoadoutEntry {
            name: name.into(),
            loadout: ScenarioTestLoadout::default(),
        }
    }

    /// The search matches the NAME only (`DoesCommandMatchSearch`,
    /// `UIDebugLoadoutsList.cs:107-114`) — unlike the commands list.
    #[test]
    fn the_search_filter_matches_the_name_only() {
        let entries = vec![entry("BossTest"), entry("arena"), entry("BossArena")];
        assert_eq!(visible_loadouts(&entries, ""), vec![0, 1, 2]);
        assert_eq!(visible_loadouts(&entries, "BOSS"), vec![0, 2]);
        assert_eq!(visible_loadouts(&entries, "  "), vec![0, 1, 2]);
        assert!(visible_loadouts(&entries, "zzz").is_empty());
    }

    /// The 70 px row lattice and the shared container geometry.
    #[test]
    fn the_row_lattice_is_70_plus_5() {
        let screen = Vec2::new(1600.0, 900.0);
        let c = container_rect(screen);
        let p = panel_rect(&c);
        let list = list_rect(&p);
        assert_eq!(row_rect(&list, 0), [412.0, 292.0, 751.0, 70.0]);
        assert_eq!(row_rect(&list, 1), [412.0, 367.0, 751.0, 70.0]);
        assert_eq!(content_height(0), 0.0);
        assert_eq!(content_height(1), 70.0);
        assert_eq!(content_height(4), 300.0);
    }

    /// `PositionItems`' group geometry (`UIDebugLoadoutItem.cs:88-91`,
    /// `:102-109`): a lone icon centres (HAlign 0.5), two icons ride the
    /// group's edges.
    #[test]
    fn the_icon_groups_follow_position_items() {
        let row = [0.0, 0.0, 100.0, 70.0];
        // Group 0: [10, 30] wide 20, vertical 0.75 → y = 52.5 + 16.
        let (x, y) = icon_center(&row, 0, 0, 1);
        assert_eq!((x, y), (20.0, 68.5));
        // Two icons: HAlign 0 → x = 10 + 16 = 26; HAlign 1 → 30 - 32 + 16
        // = 14, with `(0.3f - 0.1f) * 100` = 20.000002 in f32.
        assert_eq!(icon_center(&row, 0, 0, 2).0, 26.0);
        assert_eq!(icon_center(&row, 0, 1, 2).0, 10.0 + (0.3f32 - 0.1f32) * 100.0 - 32.0 + 16.0);
        // Group 2 (armor): vertical 0.25 → y = 17.5 + 16 = 33.5.
        assert_eq!(icon_center(&row, 2, 0, 1).1, 33.5);
    }

    /// The preview groups apply inventory THEN ammo onto the same array
    /// (the ctor's `:49-50`), and the ammo group reads indices 54..57.
    #[test]
    fn the_preview_groups_apply_both_item_lists() {
        let mut l = ScenarioTestLoadout::default();
        l.items_inventory = vec![
            ItemRecord { index: 0, kind_name: "CopperShortsword".into(), prefix: 0, stack: 1 },
            ItemRecord { index: 54, kind_name: "MusketBall".into(), prefix: 0, stack: 99 },
        ];
        l.items_ammo = vec![
            ItemRecord { index: 55, kind_name: "WoodenArrow".into(), prefix: 0, stack: 250 },
        ];
        l.items_armor = vec![
            ItemRecord { index: 0, kind_name: "CopperHelmet".into(), prefix: 0, stack: 1 },
        ];
        let groups = preview_groups(&l);
        assert_eq!(groups[0].len(), 1); // the hotbar group: slot 0 only
        assert_eq!(groups[1].len(), 2); // ammo: 54 (from inventory) + 55
        assert_eq!(groups[1][0].count, 99);
        assert_eq!(groups[1][1].count, 250);
        assert_eq!(groups[2].len(), 1); // armor head
        assert_eq!(groups[3].len(), 0); // no accessories
    }

    /// A row click resolves through the 70 px lattice with the scroll.
    #[test]
    fn the_row_click_respects_the_scroll() {
        let screen = Vec2::new(1600.0, 900.0);
        let rows = vec![0, 1, 2];
        // Row 1 at scroll 0: y 367..437.
        assert_eq!(click_at(screen, Vec2::new(500.0, 400.0), 0.0, &rows), Click::Row(1));
        // Scrolled 75: row 2 lands at 367..437.
        assert_eq!(click_at(screen, Vec2::new(500.0, 400.0), 75.0, &rows), Click::Row(2));
        assert_eq!(click_at(screen, Vec2::new(700.0, 820.0), 0.0, &rows), Click::Back);
    }
}
