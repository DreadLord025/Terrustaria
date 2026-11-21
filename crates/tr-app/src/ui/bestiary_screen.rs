//! The bestiary screen's working-set state (W4-e23): the
//! `UIBestiaryTest` machine over the [`crate::bestiary_filter`] engine —
//! `UpdateBestiaryContents` (`UIBestiaryTest.cs:432-444`), the grid's page
//! offset (`UIBestiaryEntryGrid.FixBestiaryRange`/`OffsetLibraryByPages`),
//! the sort/filter option grids' open/close arms (`:394-458`) and the
//! wrapped search bar's contents wiring (`:103-112`, `:308-312`).
//!
//! The layout rects live in [`super::bestiary`] (`top_strip`/`middle_rect`/
//! `bestiary_space`/`info_page_rect`); the draw halves in
//! [`super::bestiary_screen_draw`]. This module is state + geometry +
//! clicks, no renderer — the witness tests run headless.

use glam::Vec2;

use tr_core::bestiary::WorldBestiary;

use crate::bestiary_filter::{self as bf, EntryFilterer, EntrySorter, SortStep, SORT_STEPS};
use crate::ui::search_bar::{SearchTheme, WrappedSearchBar};

use super::bestiary as layout;

/// Which option grid overlays the bestiary space — the C# appends one of
/// the two grids as a child of `_bestiarySpace` (`:401-415`); at most one
/// is attached at a time (each open arm removes both first).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OptionsGrid {
    #[default]
    None,
    Sorting,
    Filtering,
}

/// The `UIBestiaryTest` state the port keeps on `App` — the working set,
/// the page offset, the option-grid latch and the search bar. The C#
/// rebuilds `_workingSetEntries` on every `UpdateBestiaryContents`; the
/// port stores the filtered+sorted roster indices and recomputes them at
/// the same sites (open, search change, option click, grid close).
#[derive(Clone, Debug)]
pub struct BestiaryUi {
    /// `_filterer` (`UIBestiaryTest.cs:31`).
    pub filterer: EntryFilterer,
    /// `_sorter` (`:33`).
    pub sorter: EntrySorter,
    /// `_workingSetEntries` (`:27`) — indices into `ENTRIES`, filtered then
    /// sorted (`FilterEntries`/`SortEntries`, `:314-333`).
    pub working: Vec<usize>,
    /// `_entryGrid._atEntryIndex` (`UIBestiaryEntryGrid.cs:15`) — the page
    /// start into `working`.
    pub at: usize,
    /// The attached option grid (`_sortingGrid.Parent != null` /
    /// `_filteringGrid.Parent != null`, `:396`/`:408`).
    pub options: OptionsGrid,
    /// `_sortingGrid._currentSelected` (`UIBestiarySortingOptionsGrid.cs:18`)
    /// — the last clicked STEP index (not the prioritized one: a re-click
    /// falls back to `_defaultStepIndex` 0 while this stays the clicked id).
    pub sort_selected: i32,
    /// The wrapped search bar (`:103-112`) — the Blue theme, the
    /// `UI.PlayerNameSlot` placeholder (`UIWrappedSearchBar.cs:80`).
    pub search: WrappedSearchBar,
    /// Whether `OnOpenPage` has built the working set for the current open
    /// (the env-driven `TR_BESTIARY` open and any future open path rebuild
    /// through the frame gate; the click-open calls `on_open` directly).
    pub opened: bool,
    /// The info-page scrollbar's `ViewPosition`
    /// (`UIBestiaryEntryInfoPage._scrollbar`, `UIScrollbar.cs:17`) — the
    /// wheel's `UpdateScrollbar` (`:59-65`) and the thumb drag write it;
    /// the draw clamps it through `SetView` (`UIScrollbar.cs:72-78`).
    /// W4-e23 remainder (the scrollbar thumb).
    pub info_scroll: f32,
    /// The thumb drag's `_dragYOffset` (`UIScrollbar.cs:27`) — `Some` while
    /// `_isDragging` (`:23`), cleared on the left release (`:145-149`).
    pub info_scroll_drag: Option<f32>,
}

impl Default for BestiaryUi {
    fn default() -> Self {
        Self {
            filterer: EntryFilterer::default(),
            sorter: EntrySorter::default(),
            working: Vec::new(),
            at: 0,
            options: OptionsGrid::None,
            sort_selected: -1, // `:18`
            search: WrappedSearchBar::new(&crate::lang::text("UI.PlayerNameSlot"), SearchTheme::Blue),
            opened: false,
            info_scroll: 0.0,
            info_scroll_drag: None,
        }
    }
}

impl BestiaryUi {
    /// `UpdateBestiaryContents` (`UIBestiaryTest.cs:432-444`): the
    /// availability refresh + label texts ride the draw; the state half is
    /// filter → sort → fill. `FillBestiarySpaceWithEntries` (`:335-343`)
    /// deselects the entry button (the caller clears
    /// `App::bestiary_selected` — the port keeps the selection on App) and
    /// re-clamps the page (`UIBestiaryEntryGrid.FillBestiarySpaceWithEntries:37-42`
    /// runs `FixBestiaryRange(0, capacity)`).
    pub fn update_contents(&mut self, tracker: &WorldBestiary, screen: Vec2) {
        self.working = bf::working_set(tracker, &self.filterer, &self.sorter);
        let capacity = grid_capacity(screen);
        self.at = layout::fix_grid_range(self.at, 0, capacity, self.working.len());
    }

    /// `OnOpenPage` (`:63-66`): the open rebuilds the contents.
    pub fn on_open(&mut self, tracker: &WorldBestiary, screen: Vec2) {
        self.opened = true;
        self.update_contents(tracker, screen);
    }

    /// `OnSearchContentsChanged` (`:308-312`): the filterer takes the raw
    /// payload, then the contents rebuild.
    pub fn on_search_changed(&mut self, contents: &str, tracker: &WorldBestiary, screen: Vec2) {
        self.filterer.set_search(contents);
        self.update_contents(tracker, screen);
    }

    /// `MakeButtonGoByOffset`/`OffsetLibraryByPages`
    /// (`UIBestiaryEntryGrid.cs:95-107`): ±1 page.
    pub fn page(&mut self, pages: i64, screen: Vec2) {
        let capacity = grid_capacity(screen);
        self.at =
            layout::offset_library_by_pages(self.at, pages, capacity, self.working.len());
    }

    /// `TryMovingPages` (`UIBestiaryTest.cs:852-855`) — the gamepad/wheel
    /// page step, the same offset arm.
    pub fn try_moving_pages(&mut self, direction: i64, screen: Vec2) {
        self.page(direction, screen);
    }

    /// `OpenOrCloseSortingOptions` (`:394-404`): attached → close (which
    /// rebuilds the contents, `CloseSortingGrid:454-458`); else attach,
    /// removing the filtering grid first.
    pub fn toggle_sorting(&mut self, tracker: &WorldBestiary, screen: Vec2) {
        if self.options == OptionsGrid::Sorting {
            self.options = OptionsGrid::None;
            self.update_contents(tracker, screen); // `CloseSortingGrid`
        } else {
            self.options = OptionsGrid::Sorting;
        }
    }

    /// `OpenOrCloseFilteringGrid` (`:406-416`) + `CloseFilteringGrid`
    /// (`:426-430`).
    pub fn toggle_filtering(&mut self, tracker: &WorldBestiary, screen: Vec2) {
        if self.options == OptionsGrid::Filtering {
            self.options = OptionsGrid::None;
            self.update_contents(tracker, screen);
        } else {
            self.options = OptionsGrid::Filtering;
        }
    }

    /// `Click_CloseSortingGrid`/`Click_CloseFilteringGrid` (`:418-424`,
    /// `:446-452`): a click whose TARGET is the grid panel itself (not an
    /// option button) closes it. The caller resolves the target.
    pub fn close_options_on_panel_click(&mut self, tracker: &WorldBestiary, screen: Vec2) {
        if self.options != OptionsGrid::None {
            self.options = OptionsGrid::None;
            self.update_contents(tracker, screen);
        }
    }

    /// `UIBestiarySortingOptionsGrid.ClickOption` (`:89-115`): the re-click
    /// fallback (`sort_click_option`), then `SetPrioritizedStepIndex` +
    /// `OnClickingOption` → `UpdateBestiaryContents` (`:150`).
    pub fn click_sort_option(&mut self, step_index: i32, tracker: &WorldBestiary, screen: Vec2) {
        let num = layout::sort_click_option(step_index, self.sort_selected, 0);
        self.sort_selected = num;
        self.sorter.set_prioritized(num as usize);
        self.update_contents(tracker, screen);
    }

    /// `UIBestiaryFilteringOptionsGrid.ClickOption` (`:226-235`):
    /// `ToggleFilter` + `OnClickingOption` → `UpdateBestiaryContents`
    /// (`:153`). The selection colours re-derive from `filterer.is_active`.
    pub fn click_filter_option(&mut self, filter_index: usize, tracker: &WorldBestiary, screen: Vec2) {
        self.filterer.toggle(filter_index);
        self.update_contents(tracker, screen);
    }

    /// The working-set entry under a grid slot, or `None` past the page's
    /// end (`FillBestiarySpaceWithEntries`' `num2 >= list.Count` break,
    /// `UIBestiaryEntryGrid.cs:57-60`).
    pub fn entry_at_slot(&self, slot: usize) -> Option<usize> {
        self.working.get(self.at + slot).copied()
    }

    /// `GetRangeText` (`UIBestiaryEntryGrid.cs:87-93`) over the working set.
    pub fn range_text(&self, screen: Vec2) -> String {
        layout::grid_range_text(self.at, grid_capacity(screen), self.working.len())
    }
}

/// `GetEntriesToShow`'s capacity over the bestiary space
/// (`UIBestiaryEntryGrid.cs:79-85`).
pub fn grid_capacity(screen: Vec2) -> usize {
    let panel = layout::panel_rect(screen);
    let space = layout::bestiary_space(&panel);
    let (cols, rows) = layout::grid_metrics(&space);
    cols * rows
}

// --- the top strip's button geometry (UIBestiaryTest.cs:220-300) ---------------

/// The Back/Forward page buttons (`AddBackAndForwardButtons`, `:222-236`):
/// the 28x28 `Button_Back` at the strip's left edge, `Button_Forward` at
/// `Left = back.Width + 1`. Both `SetVisibility(1, 1)` — full White hovered
/// or not, the `Button_Border` frame over the base on hover
/// (`UIImageButton.cs:48-56`).
pub fn page_buttons_rects(screen: Vec2) -> ([f32; 4], [f32; 4]) {
    let strip = layout::top_strip(&layout::panel_rect(screen));
    let back = [strip[0], strip[1], 28.0, 28.0];
    let forward = [strip[0] + 28.0 + 1.0, strip[1], 28.0, 28.0];
    (back, forward)
}

/// The range panel (`:237-254`): `Left = 28 + 1 + 28 + 3 = 60`,
/// `Width 135`, `Height (0, 1)`, `VAlign 0.5`, bg/border `(35, 40, 83)`,
/// padding 0; the range `UIText` centres in it at scale 0.8.
pub fn range_panel_rect(screen: Vec2) -> [f32; 4] {
    let strip = layout::top_strip(&layout::panel_rect(screen));
    let h = strip[3];
    [strip[0] + 60.0, strip[1] + (strip[3] - h) * 0.5, 135.0, h]
}

/// The filter/sort toggle buttons (`AddSortAndFilterButtons`, `:257-300`):
/// both 114x28, `HAlign 1`; the filtering button's
/// `Left = -infoW - 17 = -247`, the sorting button's
/// `Left = -infoW - 114 - 3 - 17 = -364` (`num = 17`, `:259`).
pub fn sort_filter_buttons_rects(screen: Vec2) -> (/*sort*/ [f32; 4], /*filter*/ [f32; 4]) {
    let strip = layout::top_strip(&layout::panel_rect(screen));
    let right = strip[0] + strip[2];
    let filter = [right - 247.0, strip[1], 114.0, 28.0];
    let sort = [right - 364.0, strip[1], 114.0, 28.0];
    (sort, filter)
}

/// The buttons' label `UIText` position (`:270-277`/`:290-297`):
/// `Left 34`, `Top 2`, `VAlign 0.5`, `TextOriginX/Y 0`, scale 0.8 — the
/// text's top sits at `button.y + 2 + (28 - textH) / 2`.
pub fn button_label_pos(button: [f32; 4], text_h: f32) -> [f32; 2] {
    [button[0] + 34.0, button[1] + 2.0 + (button[3] - text_h) * 0.5]
}

/// The wrapped search bar (`:103-112`): `Width = infoW = 230`, `HAlign 1`
/// in the top strip, 24 px tall (`UIWrappedSearchBar.cs:81`).
pub fn search_bar_rect(screen: Vec2) -> [f32; 4] {
    let strip = layout::top_strip(&layout::panel_rect(screen));
    [strip[0] + strip[2] - 230.0, strip[1], 230.0, 24.0]
}

/// The search bar's three pieces (`UIWrappedSearchBar.AddSearchBar`,
/// `:95-150`): the 22x22 magnifier at the left (`VAlign 0.5`), the box
/// panel `Width (-22 - 3, 1)` right-aligned, the 24x24 cancel button
/// inside the box at `HAlign 1`, `Left -2`, `VAlign 0.5`.
pub fn search_pieces(bar: [f32; 4]) -> (/*button*/ [f32; 4], /*box*/ [f32; 4], /*cancel*/ [f32; 4]) {
    let button = [bar[0], bar[1] + (bar[3] - 22.0) * 0.5, 22.0, 22.0];
    let box_w = bar[2] - 22.0 - 3.0;
    let bx = bar[0] + bar[2] - box_w;
    let panel = [bx, bar[1], box_w, bar[3]];
    let cancel = [
        bx + box_w - 24.0 - 2.0,
        bar[1] + (bar[3] - 24.0) * 0.5,
        24.0,
        24.0,
    ];
    (button, panel, cancel)
}

// --- the option grids' geometry -------------------------------------------------

/// The sorting grid's container (`UIBestiarySortingOptionsGrid.BuildGrid`,
/// `:49-57`): 126 wide, `visibleCount * 28 + 8` tall, `HAlign 1`,
/// `Left -118` inside the bestiary space — the container's right edge sits
/// 118 px left of the space's right edge (over the grid, beside the info
/// page).
pub fn sort_grid_rect(screen: Vec2) -> [f32; 4] {
    let space = layout::bestiary_space(&layout::panel_rect(screen));
    let visible = SORT_STEPS.iter().filter(|s| !bf::step_hidden(**s)).count();
    let size = layout::sort_grid_container_size(visible);
    [
        space[0] + space[2] - 118.0 - size[0],
        space[1],
        size[0],
        size[1],
    ]
}

/// The visible sort steps in registration order — the button rows'
/// `OptionValue`s are the STEP indices (`BuildGrid:63-81` keeps `j`).
pub fn visible_sort_steps() -> Vec<usize> {
    SORT_STEPS
        .iter()
        .enumerate()
        .filter(|(_, s)| !bf::step_hidden(**s))
        .map(|(i, _)| i)
        .collect()
}

/// The filtering grid's container (`UIBestiaryFilteringOptionsGrid.
/// BuildContainer`, `:43-59`): `perRow * 34 + 10` wide, `rows * 34 + 10`
/// tall, `HAlign 1`, `VAlign 0`, `Left/Top 0` — flush with the space's
/// top-right corner.
pub fn filter_grid_rect(screen: Vec2) -> [f32; 4] {
    let space = layout::bestiary_space(&layout::panel_rect(screen));
    let size = layout::filter_grid_container_size(bf::FILTER_COUNT);
    [space[0] + space[2] - size[0], space[1], size[0], size[1]]
}

/// Filter button `i`'s rect inside the filtering container
/// (`UpdateAvailability:95-103`): 32x32 at the shared lattice.
pub fn filter_button_rect(screen: Vec2, i: usize) -> [f32; 4] {
    let container = filter_grid_rect(screen);
    let tl = layout::filter_button_top_left(i);
    [container[0] + tl[0], container[1] + tl[1], 32.0, 32.0]
}

/// The option-grid panel click target test — a press inside the container
/// but on NO option button targets the grid itself
/// (`Click_CloseSortingGrid`'s `evt.Target == this._sortingGrid`,
/// `UIBestiaryTest.cs:446-452`).
pub fn options_panel_hit(screen: Vec2, mouse: Vec2, options: OptionsGrid) -> bool {
    match options {
        OptionsGrid::None => false,
        OptionsGrid::Sorting => {
            let container = sort_grid_rect(screen);
            if !super::in_rect(mouse, container) {
                return false;
            }
            let visible = visible_sort_steps();
            !visible
                .iter()
                .enumerate()
                .any(|(n, _)| super::in_rect(mouse, layout::sort_button_rect(container, n)))
        }
        OptionsGrid::Filtering => {
            let container = filter_grid_rect(screen);
            if !super::in_rect(mouse, container) {
                return false;
            }
            !(0..bf::FILTER_COUNT).any(|i| super::in_rect(mouse, filter_button_rect(screen, i)))
        }
    }
}

/// The sort step under the cursor, as its STEP index (`ClickOption`'s
/// `OptionValue`, `UIBestiarySortingOptionsGrid.cs:91`).
pub fn sort_step_at(screen: Vec2, mouse: Vec2) -> Option<usize> {
    let container = sort_grid_rect(screen);
    visible_sort_steps().into_iter().enumerate().find_map(|(n, step)| {
        super::in_rect(mouse, layout::sort_button_rect(container, n)).then_some(step)
    })
}

/// The filter under the cursor, as its `AvailableFilters` index. Only
/// AVAILABLE filters carry a click handler (`UpdateAvailability:104` —
/// the unavailable `-2` buttons never append `ClickOption`).
pub fn filter_at_cursor(screen: Vec2, mouse: Vec2, tracker: &WorldBestiary) -> Option<usize> {
    (0..bf::FILTER_COUNT).find(|&i| {
        super::in_rect(mouse, filter_button_rect(screen, i)) && bf::filter_available(i, tracker)
    })
}

/// The `SortStep` enum row for a step index (the label reads).
pub fn step_of(index: usize) -> SortStep {
    SORT_STEPS[index]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bestiary::ENTRIES;

    const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    /// One test's localization load — the manager is process-global and
    /// idempotent for the same culture (`lang::init`), the test_lock keeps
    /// the culture-switching tests from racing these reads.
    fn lang_ready() -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::lang::test_lock().lock().unwrap();
        crate::lang::init(tr_core::localization::GameCulture::English);
        guard
    }

    #[test]
    fn the_top_strip_lays_out_the_six_controls() {
        let _lang = lang_ready();
        // At 1280x720 the panel is [190, 180, 900, 500]; the inner panel
        // loses 90 px height, the (12, 8) padding puts the strip at
        // (202, 188) 876 wide.
        let panel = layout::panel_rect(SCREEN);
        let strip = layout::top_strip(&panel);
        assert_eq!(strip, [202.0, 188.0, 876.0, 24.0]);
        let (back, forward) = page_buttons_rects(SCREEN);
        assert_eq!(back, [202.0, 188.0, 28.0, 28.0]);
        assert_eq!(forward, [231.0, 188.0, 28.0, 28.0]);
        assert_eq!(range_panel_rect(SCREEN), [262.0, 188.0, 135.0, 24.0]);
        // The sort/filter buttons ride HAlign 1 with the -364/-247 offsets.
        let (sort, filter) = sort_filter_buttons_rects(SCREEN);
        assert_eq!(filter, [831.0, 188.0, 114.0, 28.0]);
        assert_eq!(sort, [714.0, 188.0, 114.0, 28.0]);
        // The search bar fills the info page's 230 px at the right edge.
        assert_eq!(search_bar_rect(SCREEN), [848.0, 188.0, 230.0, 24.0]);
        let (button, box_rect, cancel) = search_pieces(search_bar_rect(SCREEN));
        assert_eq!(button, [848.0, 189.0, 22.0, 22.0]);
        assert_eq!(box_rect, [873.0, 188.0, 205.0, 24.0]);
        assert_eq!(cancel, [1052.0, 188.0, 24.0, 24.0]);
        // The filter button overlaps the search bar's magnifier column
        // (both at x 831..945 / 848..870) — the C# appends the search bar
        // AFTER the buttons (`BuildPage:102-112` then `:147`), so it draws
        // over and takes the click first. The port's click order matches:
        // the search pieces are tested before the sort/filter toggles.
        assert!(filter[0] < button[0] + button[2]);
    }

    #[test]
    fn the_grid_pages_instead_of_growing() {
        let _lang = lang_ready();
        // The bestiary space at 1280x720: inner panel h = 500 - 90 = 410,
        // minus the (8, 8) padding = 394; middle h = 394 - 50 = 344,
        // space h = 340, w = 876 - 242 = 634 -> 8 cols x 4 rows = 32/page.
        let panel = layout::panel_rect(SCREEN);
        let space = layout::bestiary_space(&panel);
        assert_eq!(space[2], 634.0);
        assert_eq!(space[3], 340.0);
        let (cols, rows) = layout::grid_metrics(&space);
        assert_eq!((cols, rows), (8, 4));
        assert_eq!(grid_capacity(SCREEN), 32);
        // 241 entries page: the last page starts at 241 - 32 = 209.
        let mut ui = BestiaryUi::default();
        let tracker = WorldBestiary::new();
        ui.on_open(&tracker, SCREEN);
        assert_eq!(ui.working.len(), ENTRIES.len());
        assert_eq!(ui.at, 0);
        ui.page(100, SCREEN);
        assert_eq!(ui.at, ENTRIES.len() - 32);
        ui.page(-1000, SCREEN);
        assert_eq!(ui.at, 0);
        // The range text reads the working set.
        assert_eq!(ui.range_text(SCREEN), format!("1-32 ({})", ENTRIES.len()));
    }

    #[test]
    fn the_option_grids_are_mutually_exclusive_and_close_rebuilds() {
        let _lang = lang_ready();
        let mut ui = BestiaryUi::default();
        let tracker = WorldBestiary::new();
        ui.on_open(&tracker, SCREEN);
        ui.toggle_sorting(&tracker, SCREEN);
        assert_eq!(ui.options, OptionsGrid::Sorting);
        // Opening the filtering grid removes the sorting one (:413-415).
        ui.toggle_filtering(&tracker, SCREEN);
        assert_eq!(ui.options, OptionsGrid::Filtering);
        // A filter click rebuilds the working set immediately (:153).
        ui.click_filter_option(1, &tracker, SCREEN); // Boss
        assert!(ui.filterer.is_active(1));
        assert!(ui.working.iter().all(|&i| bf::is_boss_entry(ENTRIES[i].net_id)));
        // Closing rebuilds again (:426-430) — the filter stays active.
        ui.toggle_filtering(&tracker, SCREEN);
        assert_eq!(ui.options, OptionsGrid::None);
        assert!(ui.filterer.is_active(1));
        assert!(!ui.working.is_empty());
    }

    #[test]
    fn the_sort_reclick_falls_back_to_the_default() {
        let _lang = lang_ready();
        // ClickOption (:89-115): clicking the selected step prioritizes the
        // default 0, but _currentSelected takes the FALLBACK value too
        // (`this._currentSelected = num`, :109) — a second re-click of the
        // same button then selects it again (num != currentSelected).
        let mut ui = BestiaryUi::default();
        let tracker = WorldBestiary::new();
        ui.on_open(&tracker, SCREEN);
        ui.click_sort_option(4, &tracker, SCREEN); // ByAttack
        assert_eq!(ui.sort_selected, 4);
        assert_eq!(ui.sorter.prioritized, 4);
        ui.click_sort_option(4, &tracker, SCREEN); // re-click -> default
        assert_eq!(ui.sort_selected, 0);
        assert_eq!(ui.sorter.prioritized, 0);
        ui.click_sort_option(4, &tracker, SCREEN); // 4 != 0 -> selects again
        assert_eq!(ui.sorter.prioritized, 4);
    }

    #[test]
    fn the_search_change_rebuilds_and_the_whitespace_clears() {
        let _lang = lang_ready();
        let mut ui = BestiaryUi::default();
        let tracker = WorldBestiary::new();
        ui.on_open(&tracker, SCREEN);
        // Nothing is known in a fresh world — every provider gates on the
        // tier, so a non-empty search empties the working set.
        ui.on_search_changed("zombie", &tracker, SCREEN);
        assert!(ui.working.is_empty());
        assert_eq!(ui.range_text(SCREEN), "0-0 (0)");
        // Whitespace clears the filter object (EntryFilterer.cs:87-91).
        ui.on_search_changed("   ", &tracker, SCREEN);
        assert_eq!(ui.working.len(), ENTRIES.len());
    }

    #[test]
    fn the_option_panel_hit_skips_the_buttons() {
        let _lang = lang_ready();
        let mut ui = BestiaryUi::default();
        let tracker = WorldBestiary::new();
        ui.on_open(&tracker, SCREEN);
        ui.toggle_sorting(&tracker, SCREEN);
        let container = sort_grid_rect(SCREEN);
        // The container's top-left corner is panel, not a button (the
        // first button sits at Top 5, HAlign 0.5).
        assert!(options_panel_hit(SCREEN, Vec2::new(container[0] + 1.0, container[1] + 1.0), ui.options));
        // A button centre is NOT the panel.
        let b = layout::sort_button_rect(container, 0);
        assert!(!options_panel_hit(SCREEN, Vec2::new(b[0] + 10.0, b[1] + 10.0), ui.options));
        assert_eq!(sort_step_at(SCREEN, Vec2::new(b[0] + 10.0, b[1] + 10.0)), Some(1));
        // The filtering grid: button 0's centre hits filter 0 (forced
        // available), the container's 1 px corner is the panel.
        ui.toggle_filtering(&tracker, SCREEN);
        let fc = filter_grid_rect(SCREEN);
        assert!(options_panel_hit(SCREEN, Vec2::new(fc[0] + 1.0, fc[1] + 1.0), ui.options));
        let f0 = filter_button_rect(SCREEN, 0);
        assert_eq!(filter_at_cursor(SCREEN, Vec2::new(f0[0] + 16.0, f0[1] + 16.0), &tracker), Some(0));
        // An unavailable filter (Boss in a fresh world) has no handler.
        let f1 = filter_button_rect(SCREEN, 1);
        assert_eq!(filter_at_cursor(SCREEN, Vec2::new(f1[0] + 16.0, f1[1] + 16.0), &tracker), None);
    }

    #[test]
    fn the_info_page_sits_beside_the_grid() {
        let _lang = lang_ready();
        // The side-by-side layout: the space's right edge + 12 = the info
        // page's left edge; the page is 12 px TALLER than the middle.
        let panel = layout::panel_rect(SCREEN);
        let space = layout::bestiary_space(&panel);
        let info = layout::info_page_rect(&panel);
        assert_eq!(info[0], space[0] + space[2] + 12.0);
        assert_eq!(info[2], 230.0);
        let mid = layout::middle_rect(&panel);
        assert_eq!(info[3], mid[3] + 12.0);
        assert_eq!(info[1], mid[1]);
    }
}
