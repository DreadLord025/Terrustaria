//! The bestiary screen's hover-sound ticks (W4-e23 remainder 4): the
//! per-hover `SoundEngine.PlaySound(12)` edges the immediate draw did not
//! tick. The d81/d82 structs pin the latch
//! (`elements.rs::GroupOptionButtonState::hover_sound_tick`,
//! `GroupOptionButton.cs:212-223`); this module wires it to the screen's
//! controls.
//!
//! Two latch shapes, both the C#'s:
//! * the **draw-side one-shot** (`GroupOptionButton.DrawSelf:212-223`):
//!   while hovered the first draw sounds and latches, every later hovered
//!   draw is silent, an unhovered draw re-arms. The option-grid buttons use
//!   it (their `_soundedHover` is per-button, so moving between two buttons
//!   sounds each).
//! * the **MouseOver edge** (`UIImageButton.MouseOver:58-62`,
//!   `UIBestiaryEntryButton.MouseOver:150-152`,
//!   `UIWrappedSearchBar.searchCancelButton_OnMouseOver:166-169`): the sound
//!   fires once when the pointer enters the element. The port models it as
//!   a hovered-last-frame flag — the rising edge of the hover test is the
//!   MouseOver event.
//!
//! The state lives on [`super::bestiary_screen::BestiaryUi`]; the App ticks
//! it once per frame while the screen is open (the `GroupOptionButton`
//! DrawSelf cadence) and plays the returned cue.

use glam::Vec2;

use tr_core::bestiary::WorldBestiary;

use crate::bestiary_filter as bf;
use crate::ui::elements::GroupOptionButtonState;

use super::bestiary as layout;
use super::bestiary_screen::{self as screen, OptionsGrid};

/// The screen's hover-sound latches. The option grids keep one
/// `_soundedHover` per button (`GroupOptionButton.cs:38`); the fixed
/// controls (page/sort/filter/search buttons, the grid card) keep a
/// hovered-last-frame flag whose rising edge is the `MouseOver` event.
#[derive(Clone, Debug, Default)]
pub struct BestiaryHoverSounds {
    /// The sorting grid's per-button `_soundedHover` (the visible steps,
    /// `UIBestiarySortingOptionsGrid.BuildGrid:63-81`).
    pub sort_sounded: Vec<bool>,
    /// The filtering grid's per-button `_soundedHover`
    /// (`UIBestiaryFilteringOptionsGrid`, all `FILTER_COUNT` buttons —
    /// the unavailable ones still draw and tick, `:118-146`).
    pub filter_sounded: Vec<bool>,
    /// The Back page button's hovered-last-frame (`UIImageButton.MouseOver`).
    pub page_back: bool,
    /// The Forward page button's hovered-last-frame.
    pub page_forward: bool,
    /// The sort toggle's hovered-last-frame.
    pub sort_toggle: bool,
    /// The filter toggle's hovered-last-frame.
    pub filter_toggle: bool,
    /// The search magnifier's hovered-last-frame.
    pub search_button: bool,
    /// The search cancel button's hovered-last-frame
    /// (`searchCancelButton_OnMouseOver:166-169`).
    pub search_cancel: bool,
    /// The hovered grid card's working-set slot last frame
    /// (`UIBestiaryEntryButton.MouseOver:150-152` sounds on entry; moving
    /// to a different card is a fresh MouseOver). `None` = no card hovered.
    pub card_slot: Option<usize>,
}

impl BestiaryHoverSounds {
    /// Size the option-grid latch vecs to the button counts (the sort grid
    /// to the visible steps, the filter grid to `FILTER_COUNT`). Idempotent.
    pub fn ensure_sizes(&mut self) {
        let sort_n = screen::visible_sort_steps().len();
        if self.sort_sounded.len() != sort_n {
            self.sort_sounded.resize(sort_n, false);
        }
        if self.filter_sounded.len() != bf::FILTER_COUNT {
            self.filter_sounded.resize(bf::FILTER_COUNT, false);
        }
    }

    /// Clear every latch (the screen closed / the option grids rebuilt —
    /// the C#'s buttons are fresh objects with `_soundedHover == false`).
    pub fn reset(&mut self) {
        for b in self.sort_sounded.iter_mut() {
            *b = false;
        }
        for b in self.filter_sounded.iter_mut() {
            *b = false;
        }
        self.page_back = false;
        self.page_forward = false;
        self.sort_toggle = false;
        self.filter_toggle = false;
        self.search_button = false;
        self.search_cancel = false;
        self.card_slot = None;
    }
}

/// The rising-edge hover test — the `MouseOver` event model: sound when the
/// element is hovered now and was not last frame. Returns whether to play
/// and stores the new hovered state.
fn mouse_over_edge(hovered: bool, last: &mut bool) -> bool {
    let play = hovered && !*last;
    *last = hovered;
    play
}

/// Tick the screen's hover sounds for one frame at `mouse`. Returns `true`
/// when any control fired its hover cue (the caller plays sound 12 once —
/// the C# fires one `PlaySound(12)` per hovered element, but at most one
/// element is hovered at a time except the option-grid/card overlap, which
/// the C# resolves to a single target; the port's single cue per frame is
/// the documented immediate-mode stand-in).
///
/// `options` is the attached option grid; the grid buttons tick only while
/// their grid is attached (the C# draws only the attached grid). The fixed
/// controls tick every open frame.
pub fn tick(
    sounds: &mut BestiaryHoverSounds,
    screen_size: Vec2,
    mouse: Vec2,
    options: OptionsGrid,
    tracker: &WorldBestiary,
    working: &[usize],
    at: usize,
) -> bool {
    sounds.ensure_sizes();
    let mut play = false;

    // The option-grid buttons (`GroupOptionButton.DrawSelf:212-223`): the
    // per-button draw-side one-shot. Only the attached grid draws.
    match options {
        OptionsGrid::Sorting => {
            let container = screen::sort_grid_rect(screen_size);
            for (n, _step) in screen::visible_sort_steps().into_iter().enumerate() {
                let rect = layout::sort_button_rect(container, n);
                let hovered = crate::ui::in_rect(mouse, rect);
                if GroupOptionButtonState::hover_sound_tick(hovered, &mut sounds.sort_sounded[n]) {
                    play = true;
                }
            }
            // The filtering grid is detached — its latches re-arm.
            for b in sounds.filter_sounded.iter_mut() {
                *b = false;
            }
        }
        OptionsGrid::Filtering => {
            for i in 0..bf::FILTER_COUNT {
                let rect = screen::filter_button_rect(screen_size, i);
                let hovered = crate::ui::in_rect(mouse, rect);
                if GroupOptionButtonState::hover_sound_tick(hovered, &mut sounds.filter_sounded[i]) {
                    play = true;
                }
            }
            for b in sounds.sort_sounded.iter_mut() {
                *b = false;
            }
        }
        OptionsGrid::None => {
            for b in sounds.sort_sounded.iter_mut() {
                *b = false;
            }
            for b in sounds.filter_sounded.iter_mut() {
                *b = false;
            }
        }
    }

    // The fixed controls (`UIImageButton.MouseOver:58-62`): the rising edge.
    let (back, forward) = screen::page_buttons_rects(screen_size);
    if mouse_over_edge(crate::ui::in_rect(mouse, back), &mut sounds.page_back) {
        play = true;
    }
    if mouse_over_edge(crate::ui::in_rect(mouse, forward), &mut sounds.page_forward) {
        play = true;
    }
    let (sort_btn, filter_btn) = screen::sort_filter_buttons_rects(screen_size);
    if mouse_over_edge(crate::ui::in_rect(mouse, sort_btn), &mut sounds.sort_toggle) {
        play = true;
    }
    if mouse_over_edge(crate::ui::in_rect(mouse, filter_btn), &mut sounds.filter_toggle) {
        play = true;
    }
    let bar = screen::search_bar_rect(screen_size);
    let (button, _box, cancel) = screen::search_pieces(bar);
    if mouse_over_edge(crate::ui::in_rect(mouse, button), &mut sounds.search_button) {
        play = true;
    }
    // The cancel button's own OnMouseOver (`UIWrappedSearchBar.cs:166-169`).
    if mouse_over_edge(crate::ui::in_rect(mouse, cancel), &mut sounds.search_cancel) {
        play = true;
    }

    // The grid card (`UIBestiaryEntryButton.MouseOver:150-152`): sound when
    // the hovered card changes (a fresh MouseOver). The slot maps through
    // the page start to the working-set entry; an empty slot is no card.
    let hovered_slot = layout::grid_slot_at(screen_size, mouse).filter(|&s| {
        at + s < working.len()
    });
    if hovered_slot != sounds.card_slot {
        if hovered_slot.is_some() {
            play = true;
        }
        sounds.card_slot = hovered_slot;
    }
    let _ = tracker;
    play
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Vec2 = Vec2::new(1280.0, 720.0);

    /// The MouseOver edge: sound on the rising edge only.
    #[test]
    fn the_mouse_over_edge_fires_once_per_entry() {
        let mut last = false;
        assert!(mouse_over_edge(true, &mut last)); // enter
        assert!(!mouse_over_edge(true, &mut last)); // stay
        assert!(!mouse_over_edge(false, &mut last)); // leave
        assert!(mouse_over_edge(true, &mut last)); // re-enter
    }

    /// The option-button latch: the d81 one-shot, per button.
    #[test]
    fn the_option_button_latch_is_per_button() {
        let mut a = false;
        let mut b = false;
        // Hovering button A sounds it; B stays silent.
        assert!(GroupOptionButtonState::hover_sound_tick(true, &mut a));
        assert!(!GroupOptionButtonState::hover_sound_tick(false, &mut b));
        // Moving to B sounds B (A re-arms).
        assert!(!GroupOptionButtonState::hover_sound_tick(false, &mut a));
        assert!(GroupOptionButtonState::hover_sound_tick(true, &mut b));
        // Back to A sounds again.
        assert!(GroupOptionButtonState::hover_sound_tick(true, &mut a));
    }

    /// A card change is a fresh MouseOver; staying on the same card is not.
    #[test]
    fn the_card_hover_sounds_on_change_only() {
        let mut sounds = BestiaryHoverSounds::default();
        let tracker = WorldBestiary::new();
        let working: Vec<usize> = (0..32).collect();
        // The first card's centre at 1280x720.
        let panel = layout::panel_rect(SCREEN);
        let space = layout::bestiary_space(&panel);
        let c0 = Vec2::new(space[0] + layout::CELL * 0.5, space[1] + layout::CELL * 0.5);
        let c1 = Vec2::new(space[0] + layout::CELL * 1.5, space[1] + layout::CELL * 0.5);
        // Enter card 0: sounds.
        assert!(tick(&mut sounds, SCREEN, c0, OptionsGrid::None, &tracker, &working, 0));
        // Stay on card 0: silent.
        assert!(!tick(&mut sounds, SCREEN, c0, OptionsGrid::None, &tracker, &working, 0));
        // Move to card 1: sounds.
        assert!(tick(&mut sounds, SCREEN, c1, OptionsGrid::None, &tracker, &working, 0));
        // Off the grid: silent, and re-entering card 1 sounds again.
        let off = Vec2::new(space[0] - 10.0, space[1] - 10.0);
        assert!(!tick(&mut sounds, SCREEN, off, OptionsGrid::None, &tracker, &working, 0));
        assert!(tick(&mut sounds, SCREEN, c1, OptionsGrid::None, &tracker, &working, 0));
    }
}
