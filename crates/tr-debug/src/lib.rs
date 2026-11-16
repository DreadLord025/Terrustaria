//! The F3 developer workbench: panel state, layout, hit-testing and the
//! input→intent translation.
//!
//! The crate deliberately knows nothing about the game. `tr-app` is a
//! **binary** crate (`[[bin]]`, no `[lib]`), so it cannot be depended upon —
//! the direction is inverted here: this crate owns the *shape* of the panel and
//! the host supplies the rows, draws them through [`Panel::layout`] and maps an
//! [`Event`] back to its own action. That is the whole seam.
//!
//! Layout (the "Hybrid Workbench": tab rail + inspector + dense list):
//!
//! ```text
//! +------------------------------------------+
//! | DEBUG [F3]                               |  title bar (drag)
//! +--------+---------------------------------+
//! | Items  | Search: ______________          |  rail | filter
//! | NPCs   | > icon  name         [id]       |  rail | row
//! | Bosses |   icon  name         [id]       |  rail | row
//! | World  |   ...                           |  rail | row
//! | Recipes|                                 |  rail |
//! +--------+---------------------------------+
//! | inspector: the selected row's details     |
//! +------------------------------------------+
//! ```

use glam::Vec2;

// ---- The layout constants ----

pub const PW: f32 = 420.0;
pub const PH: f32 = 520.0;
pub const TITLE_H: f32 = 20.0;
pub const RAIL_W: f32 = 88.0;
pub const TAB_H: f32 = 24.0;
pub const FILTER_H: f32 = 20.0;
pub const ROW_H: f32 = 22.0;
pub const PAD: f32 = 8.0;
pub const FOOTER_H: f32 = 40.0;
pub const ICON: f32 = 18.0;

// ---- The tabs ----

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Tab {
    #[default]
    Items,
    Npcs,
    Bosses,
    World,
    Recipes,
}

pub const TABS: [(&str, Tab); 5] = [
    ("Items", Tab::Items),
    ("NPCs", Tab::Npcs),
    ("Bosses", Tab::Bosses),
    ("World", Tab::World),
    ("Recipes", Tab::Recipes),
];

impl Tab {
    /// The index into [`TABS`] (the rail's row).
    pub fn index(self) -> usize {
        TABS.iter().position(|(_, t)| *t == self).unwrap_or(0)
    }
}

// ---- The intents ----

/// What the host must act on. The panel never mutates game state, so every
/// gameplay consequence goes through this enum.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    /// The panel opened or closed (the F3 toggle).
    Toggled(bool),
    /// The user picked a tab.
    TabSelected(Tab),
    /// A row of `tab` was activated. `row` is the absolute index into the
    /// host's list for that tab, already unscrolled.
    RowClicked { tab: Tab, row: usize },
    /// The filter text changed (the host re-filters its rows).
    FilterChanged,
    /// The scroll offset changed (the host may redraw a scrollbar).
    Scrolled,
}

// ---- The state ----

#[derive(Clone, Debug)]
pub struct Panel {
    pub open: bool,
    pub tab: Tab,
    /// The scroll offset in rows of the current tab.
    pub scroll: f32,
    /// The search/filter text.
    pub filter: String,
    /// The panel's top-left corner (the title bar is the drag handle).
    pub pos: Vec2,
    /// The cursor minus the panel origin during a drag.
    pub drag_offset: Option<Vec2>,
    /// The keyboard-focused row (accessibility: the list is reachable without
    /// the mouse). `None` = nothing focused yet.
    pub focus: Option<usize>,
}

impl Panel {
    pub fn new() -> Self {
        Self {
            open: false,
            tab: Tab::default(),
            scroll: 0.0,
            filter: String::new(),
            pos: Vec2::new(900.0, 60.0),
            drag_offset: None,
            focus: None,
        }
    }

    /// F3: flips the panel. Closing clears the drag so a stale grab cannot
    /// resume from a hidden panel.
    pub fn toggle(&mut self) -> Event {
        self.open = !self.open;
        if !self.open {
            self.drag_offset = None;
        }
        Event::Toggled(self.open)
    }

    /// The first visible row index.
    pub fn scroll_row(&self) -> usize {
        self.scroll.max(0.0) as usize
    }

    /// How many rows fit the list at the current layout.
    pub fn visible_rows(&self) -> usize {
        let l = self.layout(Vec2::ZERO);
        (l.list[3] / ROW_H).max(0.0) as usize
    }

    /// The layout rects for the panel at its current position.
    pub fn layout(&self, _screen: Vec2) -> Layout {
        Layout::new(self.pos)
    }

    // ---- The input ----

    /// A left click at `mouse`, against a list of `rows` entries on the active
    /// tab. `rows` is only needed to clamp the hit row.
    pub fn click(&mut self, mouse: Vec2, rows: usize) -> Option<Event> {
        if !self.open {
            return None;
        }
        let l = self.layout(Vec2::ZERO);

        // The title bar: start a drag.
        if in_rect(mouse, l.title) {
            self.drag_offset = Some(Vec2::new(mouse.x - self.pos.x, mouse.y - self.pos.y));
            return None;
        }
        // The tab rail.
        for (i, (_, tab)) in TABS.iter().enumerate() {
            let r = l.tab(i);
            if in_rect(mouse, r) {
                if self.tab != *tab {
                    self.tab = *tab;
                    self.scroll = 0.0;
                    self.focus = None;
                }
                return Some(Event::TabSelected(*tab));
            }
        }
        // The list.
        if in_rect(mouse, l.list) {
            let vi = ((mouse.y - l.list[1]) / ROW_H) as usize;
            let idx = self.scroll_row() + vi;
            if vi < (l.list[3] / ROW_H) as usize && idx < rows {
                self.focus = Some(idx);
                return Some(Event::RowClicked {
                    tab: self.tab,
                    row: idx,
                });
            }
        }
        None
    }

    /// The wheel: moves the list by `delta` rows (the sign matches winit's `y`).
    /// Clamped so the last page cannot scroll past the end.
    pub fn scroll_by(&mut self, delta: f32, rows: usize) -> bool {
        if !self.open {
            return false;
        }
        let visible = self.visible_rows();
        let max = rows.saturating_sub(visible) as f32;
        let next = (self.scroll - delta * 3.0).clamp(0.0, max);
        if next == self.scroll {
            return false;
        }
        self.scroll = next;
        true
    }

    /// The drag move: the title bar is held.
    pub fn drag(&mut self, mouse: Vec2) -> bool {
        match self.drag_offset {
            Some(offset) if self.open => {
                self.pos = Vec2::new(mouse.x - offset.x, mouse.y - offset.y);
                true
            }
            _ => false,
        }
    }

    /// The release settles the drag.
    pub fn end_drag(&mut self) -> bool {
        self.drag_offset.take().is_some()
    }

    // ---- The filter ----

    pub fn push_filter(&mut self, c: char) -> Event {
        self.filter.push(c);
        self.scroll = 0.0;
        self.focus = None;
        Event::FilterChanged
    }

    pub fn pop_filter(&mut self) -> Event {
        self.filter.pop();
        self.scroll = 0.0;
        self.focus = None;
        Event::FilterChanged
    }

    // ---- The keyboard (accessibility: the list works without a mouse) ----

    /// Moves the focused row by `step`, clamped to `rows`. Selects the first row
    /// when nothing is focused yet.
    pub fn move_focus(&mut self, step: i32, rows: usize) -> bool {
        if rows == 0 {
            return false;
        }
        let next = match self.focus {
            None => 0,
            Some(i) => (i as i32 + step).clamp(0, rows as i32 - 1) as usize,
        };
        if self.focus == Some(next) {
            return false;
        }
        self.focus = Some(next);
        // Keep the focused row on screen.
        let visible = self.visible_rows().max(1);
        let top = self.scroll_row();
        if next < top {
            self.scroll = next as f32;
        } else if next >= top + visible {
            self.scroll = (next + 1 - visible) as f32;
        }
        true
    }

    /// Enter: activates the focused row.
    pub fn activate(&self, rows: usize) -> Option<Event> {
        let row = self.focus?;
        (self.open && row < rows).then_some(Event::RowClicked { tab: self.tab, row })
    }
}

impl Default for Panel {
    fn default() -> Self {
        Self::new()
    }
}

// ---- The layout ----

/// The panel's rects at a given origin.
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    /// The whole panel `[x, y, w, h]`.
    pub panel: [f32; 4],
    /// The title bar / drag handle.
    pub title: [f32; 4],
    /// The tab rail column.
    pub rail: [f32; 4],
    /// The filter field.
    pub filter: [f32; 4],
    /// The row list.
    pub list: [f32; 4],
    /// The selected-row inspector strip.
    pub footer: [f32; 4],
}

impl Layout {
    pub fn new(pos: Vec2) -> Self {
        let panel = [pos.x, pos.y, PW, PH];
        let title = [pos.x, pos.y, PW, TITLE_H];
        let body_top = pos.y + TITLE_H + PAD;
        let body_h = PH - TITLE_H - PAD * 2.0 - FOOTER_H;
        let rail = [pos.x + PAD, body_top, RAIL_W - PAD, body_h];
        let right_x = pos.x + RAIL_W + PAD;
        let right_w = PW - RAIL_W - PAD * 2.0;
        let filter = [right_x, body_top, right_w, FILTER_H];
        let list = [
            right_x,
            body_top + FILTER_H,
            right_w,
            body_h - FILTER_H,
        ];
        let footer = [
            pos.x + PAD,
            pos.y + PH - FOOTER_H,
            PW - PAD * 2.0,
            FOOTER_H - PAD,
        ];
        Self {
            panel,
            title,
            rail,
            filter,
            list,
            footer,
        }
    }

    /// The rail row of tab `i`.
    pub fn tab(&self, i: usize) -> [f32; 4] {
        [
            self.rail[0],
            self.rail[1] + i as f32 * TAB_H,
            self.rail[2],
            TAB_H,
        ]
    }

    /// The y of row `vi` (the *visible* index).
    pub fn row_y(&self, vi: usize) -> f32 {
        self.list[1] + vi as f32 * ROW_H
    }
}

/// The point-in-rect test (`UIElement`'s `Contains` shape, `x/y/w/h`).
pub fn in_rect(p: Vec2, r: [f32; 4]) -> bool {
    p.x >= r[0] && p.x <= r[0] + r[2] && p.y >= r[1] && p.y <= r[1] + r[3]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open_panel() -> Panel {
        let mut p = Panel::new();
        p.toggle();
        p.pos = Vec2::new(100.0, 50.0);
        p
    }

    #[test]
    fn the_f3_toggle_flips_and_clears_the_drag() {
        let mut p = Panel::new();
        assert!(!p.open);
        assert_eq!(p.toggle(), Event::Toggled(true));
        p.drag_offset = Some(Vec2::ONE);
        assert_eq!(p.toggle(), Event::Toggled(false));
        assert_eq!(p.drag_offset, None, "a hidden panel must not keep its grab");
    }

    #[test]
    fn the_closed_panel_swallows_every_click() {
        let p = Panel::new();
        let mut p = p;
        assert!(p.click(Vec2::new(110.0, 55.0), 100).is_none());
    }

    #[test]
    fn the_rail_picks_the_tab_and_resets_scroll_and_focus() {
        let mut p = open_panel();
        p.scroll = 12.0;
        p.focus = Some(9);
        let l = p.layout(Vec2::ZERO);
        let r = l.tab(2); // Bosses
        let mouse = Vec2::new(r[0] + 4.0, r[1] + 4.0);
        assert_eq!(p.click(mouse, 40), Some(Event::TabSelected(Tab::Bosses)));
        assert_eq!(p.tab, Tab::Bosses);
        assert_eq!(p.scroll, 0.0);
        assert_eq!(p.focus, None);
    }

    #[test]
    fn the_title_bar_starts_a_drag_and_the_move_follows() {
        let mut p = open_panel();
        let l = p.layout(Vec2::ZERO);
        let grab = Vec2::new(l.title[0] + 5.0, l.title[1] + 5.0);
        assert_eq!(p.click(grab, 10), None);
        assert!(p.drag_offset.is_some());
        assert!(p.drag(Vec2::new(grab.x + 30.0, grab.y + 40.0)));
        assert_eq!(p.pos, Vec2::new(130.0, 90.0));
        assert!(p.end_drag());
        assert!(!p.end_drag(), "the second release has nothing to settle");
    }

    #[test]
    fn a_list_click_returns_the_unscrolled_row() {
        let mut p = open_panel();
        p.scroll = 5.0;
        let l = p.layout(Vec2::ZERO);
        // The third visible row.
        let mouse = Vec2::new(l.list[0] + 4.0, l.row_y(2) + ROW_H * 0.5);
        assert_eq!(
            p.click(mouse, 100),
            Some(Event::RowClicked {
                tab: Tab::Items,
                row: 7,
            })
        );
    }

    #[test]
    fn a_click_past_the_last_row_is_ignored() {
        let mut p = open_panel();
        let l = p.layout(Vec2::ZERO);
        let mouse = Vec2::new(l.list[0] + 4.0, l.row_y(1) + ROW_H * 0.5);
        assert_eq!(p.click(mouse, 1), None, "row 1 does not exist");
        assert_eq!(p.focus, None);
    }

    #[test]
    fn the_wheel_clamps_to_the_last_page() {
        let mut p = open_panel();
        let visible = p.visible_rows();
        assert!(visible > 0);
        assert!(p.scroll_by(-10.0, 1000), "scrolling down moves the list");
        assert_eq!(p.scroll, 30.0, "three rows per wheel notch");
        // Drive it past the end.
        for _ in 0..200 {
            p.scroll_by(-10.0, 1000);
        }
        assert_eq!(p.scroll, (1000 - visible) as f32);
        assert!(!p.scroll_by(-10.0, 1000), "already at the end");
        assert!(p.scroll_by(10.0, 1000), "scrolling up moves back");
    }

    #[test]
    fn a_list_shorter_than_the_page_cannot_scroll() {
        let mut p = open_panel();
        assert!(!p.scroll_by(-10.0, 3), "nothing to scroll");
        assert_eq!(p.scroll, 0.0);
    }

    #[test]
    fn the_keyboard_reaches_the_list_without_a_mouse() {
        let mut p = open_panel();
        assert!(p.move_focus(1, 100), "the first press selects row 0");
        assert_eq!(p.focus, Some(0));
        assert!(p.move_focus(1, 100));
        assert_eq!(p.focus, Some(1));
        assert_eq!(
            p.activate(100),
            Some(Event::RowClicked {
                tab: Tab::Items,
                row: 1,
            })
        );
        // The clamp at both ends.
        p.focus = Some(99);
        assert!(!p.move_focus(1, 100));
        assert_eq!(p.focus, Some(99));
    }

    #[test]
    fn the_keyboard_scrolls_the_focused_row_into_view() {
        let mut p = open_panel();
        let visible = p.visible_rows();
        p.focus = Some(visible - 1);
        assert_eq!(p.scroll, 0.0);
        assert!(p.move_focus(1, 500), "one past the last visible row");
        assert_eq!(p.scroll, 1.0, "the view follows the focus");
        // Walk back up: the view must follow upwards too.
        p.focus = Some(0);
        p.scroll = 9.0;
        assert!(p.move_focus(-1, 500) || p.focus == Some(0));
        p.focus = Some(1);
        assert!(p.move_focus(-1, 500));
        assert_eq!(p.focus, Some(0));
        assert_eq!(p.scroll, 0.0, "the top row scrolls the view back");
    }

    #[test]
    fn the_filter_resets_the_list_position() {
        let mut p = open_panel();
        p.scroll = 40.0;
        p.focus = Some(7);
        assert_eq!(p.push_filter('a'), Event::FilterChanged);
        assert_eq!(p.filter, "a");
        assert_eq!(p.scroll, 0.0);
        assert_eq!(p.focus, None);
        assert_eq!(p.pop_filter(), Event::FilterChanged);
        assert_eq!(p.filter, "");
        p.pop_filter();
        assert_eq!(p.filter, "", "popping an empty filter is a no-op");
    }

    #[test]
    fn every_tab_has_a_rail_row() {
        for (i, (_, tab)) in TABS.iter().enumerate() {
            assert_eq!(tab.index(), i);
        }
    }

    #[test]
    fn the_layout_nests_without_overlap() {
        let l = Layout::new(Vec2::new(10.0, 20.0));
        assert_eq!(l.panel, [10.0, 20.0, PW, PH]);
        // The rail sits left of the list, the title above both.
        assert!(l.rail[0] + l.rail[2] <= l.list[0]);
        assert!(l.title[1] + l.title[3] <= l.rail[1]);
        assert!(l.filter[1] + l.filter[3] <= l.list[1]);
        assert!(l.list[1] + l.list[3] <= l.footer[1] + 0.001);
        // The rail's five tabs fit its height.
        let last = l.tab(TABS.len() - 1);
        assert!(last[1] + last[3] <= l.rail[1] + l.rail[3] + 0.001);
    }
}
