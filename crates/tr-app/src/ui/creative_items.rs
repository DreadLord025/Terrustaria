//! The Journey creative item grid + its filter strip's element halves —
//! W4-d82 (research/18b rows 23/24): `UICreativeItemGrid`
//! (`GameContent/UI/Elements/UICreativeItemGrid.cs:9-35`) and the missing
//! pieces of `UICreativeItemsInfiniteFilteringOptions`
//! (`UICreativeItemsInfiniteFilteringOptions.cs:14-133`).
//!
//! The filter *board* (the exclusive toggle) is landed as
//! `craft_filter::FilterBoard` and the strip geometry as `crafting::filter_*`;
//! this module adds the grid's slot-context pick and the strip's layout/icon
//! tint the crafting landing did not carry. The grid's slot geometry rides
//! `ui::item_collection` (the `UIDynamicItemCollection` base, row 30).

/// `UICreativeItemGrid.DrawSlot`'s context pick (`UICreativeItemGrid.cs:21`):
/// `IsFullyResearched(item.type) ? 29 : 34` — context 29 is the creative
/// infinite slot (click spawns a stack), 34 the research-preview slot.
pub fn slot_context(fully_researched: bool) -> i32 {
    if fully_researched {
        29
    } else {
        34
    }
}

/// `DrawSlot`'s hover arm (`:22-29`): while hovering, the slot's item is
/// re-`SetDefaults`ed (a fresh copy — the grid's entries are the shared
/// `ContentSamples` items), `ItemSlot.Handle` runs, and the item's
/// newly-researched status clears (`ClearNewlyResearchedStatus`, `:28`).
/// Returns `(handle, clear_newly)` — the two side effects the caller routes.
pub fn hover_effects(hovering: bool) -> (bool, bool) {
    (hovering, hovering)
}

/// `DrawSlot`'s highlight override (`:31-33`): `DrawNewAndShinyHighlightOverride
/// = IsNewlyResearched(item.type)` around the draw, reset to false after.
pub fn shiny_override(newly_researched: bool) -> bool {
    newly_researched
}

/// `UICreativeItemsInfiniteFilteringOptions.ColorTheme` (`:16-20`): the tab
/// sheet pick (`:46-50`) — Blue uses `Infinite_Tabs_B`, Cyan `Infinite_Tabs_B_2`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorTheme {
    #[default]
    Blue,
    Cyan,
}

impl ColorTheme {
    /// The ctor's asset name (`:46-50`).
    pub fn tabs_asset(self) -> &'static str {
        match self {
            ColorTheme::Blue => "Images/UI/Creative/Infinite_Tabs_B",
            ColorTheme::Cyan => "Images/UI/Creative/Infinite_Tabs_B_2",
        }
    }
}

/// The ctor's bar sizing (`:40-45`): `num = 40`, height 40 px, width
/// `40 * filter_count`, `Top = 4 px`, padding 0. Returns `(width, height, top)`.
pub fn strip_size(filter_count: usize) -> (f32, f32, f32) {
    (40.0 * filter_count as f32, 40.0, 4.0)
}

/// The per-tab button placement (`:56-57`): `Left = 40 * i`, and the frame is
/// `Frame(2, 4).OffsetSize(-2, -2)` — the same 42×48→40×46 cells the crafting
/// strip draws (`crafting::filter_button_src`), so the *element* ctor sizes the
/// button 40×46 while the strip reserves 40×40 rows.
pub fn tab_left(i: usize) -> f32 {
    40.0 * i as f32
}

/// `UpdateVisuals`' icon tint (`:107-110`): the `IColorable` icon is White when
/// the filter is active, `White * 0.5` otherwise (XNA truncation: 255·0.5 =
/// 127.5 → 127). The frame col/row pick is landed (`crafting::filter_frame_col_row`).
pub fn icon_color(active: bool) -> [u8; 4] {
    if active {
        [255, 255, 255, 255]
    } else {
        [127, 127, 127, 127]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The context pick + hover effects + shiny override (`UICreativeItemGrid.cs:21-33`).
    #[test]
    fn the_grid_slot_picks_context_and_hover_effects() {
        assert_eq!(slot_context(true), 29);
        assert_eq!(slot_context(false), 34);
        assert_eq!(hover_effects(true), (true, true));
        assert_eq!(hover_effects(false), (false, false));
        assert!(shiny_override(true));
        assert!(!shiny_override(false));
    }

    /// The strip ctor sizing + theme + icon tint
    /// (`UICreativeItemsInfiniteFilteringOptions.cs:40-50`, `:107-110`).
    #[test]
    fn the_strip_sizes_and_tints() {
        assert_eq!(strip_size(11), (440.0, 40.0, 4.0));
        assert_eq!(tab_left(3), 120.0);
        assert_eq!(ColorTheme::default(), ColorTheme::Blue);
        assert_eq!(
            ColorTheme::Blue.tabs_asset(),
            "Images/UI/Creative/Infinite_Tabs_B"
        );
        assert_eq!(
            ColorTheme::Cyan.tabs_asset(),
            "Images/UI/Creative/Infinite_Tabs_B_2"
        );
        assert_eq!(icon_color(true), [255, 255, 255, 255]);
        // White * 0.5 truncates (`ui::color::mul`).
        assert_eq!(icon_color(false), crate::ui::color::mul([255; 4], 0.5));
    }
}
