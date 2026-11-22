//! The search-bar element — W4-d83 (research/18b row 49): `UISearchBar`
//! (`GameContent/UI/Elements/UISearchBar.cs:11-197`). The crafting filter's
//! immediate draw (`crafting.rs`, the `UIWrappedSearchBar` variant, row 63)
//! pre-dates this port; here is the base bar's *state machine*: the
//! placeholder/contents swap, the display-trim loop, the input-taking latch.
//! The inner `UITextBox` (row 54) and the IME/`Main.GetInputText` plumbing
//! stay the caller's seams.

/// `Utils.TrimUserString` (`Utils.cs:2839-2850`): truncate to `length`
/// **UTF-16 code units** (C# `string.Length`), backing off one unit when the
/// cut would leave a dangling high surrogate (`:2845-2848`). Shared by the
/// search bars and `UITextBox.SetText` (`UITextBox.cs:64-68`).
pub fn trim_user_string(s: &str, length: usize) -> String {
    let units: Vec<u16> = s.encode_utf16().collect();
    if units.len() <= length {
        return s.to_string(); // `:2841-2844`
    }
    let mut length = length;
    // `char.IsHighSurrogate(s[length - 1])` — 0xD800..=0xDBFF.
    if length > 0 && (0xD800..=0xDBFF).contains(&units[length - 1]) {
        length -= 1; // `:2847`
    }
    String::from_utf16_lossy(&units[..length]) // `:2849` Substring
}

/// `Utils.TrimLastCharacter` (`Utils.cs:2852-2855`):
/// `TrimUserString(s, s.Length - 1)` in UTF-16 units — an emoji costs two.
pub fn trim_last_character(s: &str) -> String {
    let len = s.encode_utf16().count();
    trim_user_string(s, len.saturating_sub(1))
}

/// `UISearchBar` (`UISearchBar.cs:11-197`), research/18b row 49 (W4-d83).
/// The port keeps `actualContents`, the display text the inner `UITextBox`
/// shows (placeholder when empty), the 50-char cap and the input latch.
#[derive(Clone, Debug, PartialEq)]
pub struct SearchBar {
    /// `actualContents` (`:17`) — the raw query, `""` when showing the
    /// placeholder.
    pub actual_contents: String,
    /// The inner box's `Text` — the placeholder or the (trimmed) contents.
    pub display_text: String,
    /// `_textScale` (`:19`, ctor `:51`).
    pub text_scale: f32,
    /// `_maxInputLength` (`:21`) — the ctor sets 50 (`:63`).
    pub max_input_length: usize,
    /// `IsWritingText` (`:25`); `ShowInputTicker` mirrors it (`:172`).
    pub is_writing_text: bool,
    /// The ctor's empty-content label (`_textToShowWhenEmpty`, `:13`).
    pub empty_text: String,
}

impl SearchBar {
    /// The ctor (`:48-65`): empty contents, the placeholder shown in Gray,
    /// cap 50, ticker off (`ShowInputTicker = false`, `:61`).
    pub fn new(empty_text: &str, scale: f32) -> Self {
        Self {
            actual_contents: String::new(),
            display_text: empty_text.to_string(),
            text_scale: scale,
            max_input_length: 50,
            is_writing_text: false,
            empty_text: empty_text.to_string(),
        }
    }

    /// `HasContents` (`:23`): `!string.IsNullOrWhiteSpace(actualContents)`.
    pub fn has_contents(&self) -> bool {
        !self.actual_contents.trim().is_empty()
    }

    /// The inner box's `TextColor` after a `SetContents` (`:74`, `:79`):
    /// Gray while the placeholder shows, White otherwise.
    pub fn text_color(&self) -> [u8; 4] {
        if self.actual_contents.is_empty() {
            [128, 128, 128, 255] // `Color.Gray`
        } else {
            [255, 255, 255, 255]
        }
    }

    /// `SetContents` (`:67-89`): a same-string write without `forced` is a
    /// no-op (returns `false`, no `OnContentsChanged`); otherwise the
    /// contents swap, an empty string shows the placeholder in Gray, a
    /// non-empty one goes through the box's max-length trim and reads back
    /// (`:81` `actualContents = _text.Text`). Returns `true` when the
    /// change event fires. The caller runs
    /// [`Self::trim_display_if_over_dimensions`] after (`:83`).
    pub fn set_contents(&mut self, contents: &str, forced: bool) -> bool {
        if self.actual_contents == contents && !forced {
            return false; // `:69`
        }
        self.actual_contents = contents.to_string();
        if self.actual_contents.is_empty() {
            self.display_text = self.empty_text.clone(); // `:74-75`
        } else {
            // `SetText` → `Utils.TrimUserString(text, _maxLength)`
            // (`UITextBox.cs:64-68`), then the read-back (`:81`).
            self.display_text = trim_user_string(&self.actual_contents, self.max_input_length);
            self.actual_contents = self.display_text.clone();
        }
        true // `:84-87` OnContentsChanged
    }

    /// `TrimDisplayIfOverElementDimensions` (`:91-114`): while the text
    /// box's right edge (`text_x + measured MinWidth`) passes the element's
    /// right edge minus `padding`, drop the last character and re-measure;
    /// every trim writes back to `actualContents` (`:111`). Both rects are
    /// built from **int-truncated** positions/sizes (`:96-102`), so the
    /// comparison is `text_x + min_w > elem_x + elem_w - padding` in ints.
    /// `measure` returns the box's `MinWidth.Pixels` for a display text
    /// (the `SetText` minimum, `UITextPanel.cs:133`); a zero-sized element
    /// skips the loop (`:94`).
    pub fn trim_display_if_over_dimensions(
        &mut self,
        elem_rect: [f32; 4],
        text_x: f32,
        padding: i32,
        mut measure: impl FnMut(&str) -> f32,
    ) {
        if elem_rect[2] == 0.0 && elem_rect[3] == 0.0 {
            return; // `:94` — the || keeps any non-zero dimension alive
        }
        let elem_right = elem_rect[0] as i32 + elem_rect[2] as i32; // `:96-98`
        let text_x = text_x as i32; // `:100`
        while !self.display_text.is_empty() {
            let min_w = measure(&self.display_text) as i32; // `:101` cast
            if text_x + min_w <= elem_right - padding {
                break; // `:103` the while's negation
            }
            self.display_text = trim_last_character(&self.display_text); // `:105`
            self.actual_contents = self.display_text.clone(); // `:111`
        }
    }

    /// `ToggleTakingText` (`:169-188`): flip the latch, mirror the ticker,
    /// `Main.clrInput()` (the caller's), then fire exactly one of the
    /// start/end events — the end arm also drops `PlayerInput.WritingText`.
    /// Returns the new latch value.
    pub fn toggle_taking_text(&mut self) -> bool {
        self.is_writing_text = !self.is_writing_text;
        self.is_writing_text
    }

    /// `OnDeactivate` (`:190-196`): leaving the screen while writing toggles
    /// the input off; returns `true` when it fired.
    pub fn on_deactivate(&mut self) -> bool {
        if self.is_writing_text {
            self.toggle_taking_text();
            true
        } else {
            false
        }
    }
}

/// `UIWrappedSearchBar.ColorTheme` (`UIWrappedSearchBar.cs:14-18`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SearchTheme {
    #[default]
    Blue,
    Red,
}

/// The box panel's base colour (`UIWrappedSearchBar.cs:117-118`):
/// `(35, 40, 83)` for both bg and border; the Red theme shifts it through
/// `Utils.ShiftBlueToCyanTheme` (`:119-123`, `color::shift_blue_to_cyan`).
pub fn search_box_color(theme: SearchTheme) -> [u8; 4] {
    let base = [35, 40, 83, 255];
    match theme {
        SearchTheme::Blue => base,
        SearchTheme::Red => super::color::shift_blue_to_cyan(base),
    }
}

/// `UIWrappedSearchBar` (`UIWrappedSearchBar.cs:12-247`), research/18b
/// row 63 (W4-d83): the magnifier-button + box-panel + cancel-button
/// wrapper around a [`SearchBar`]. The crafting filter's immediate draw is
/// landed (`crafting.rs:756-838`, the Red theme + geometry); this carries
/// the wrapper's state machine: the input-focus border swap, the cancel /
/// right-click / outside-click arms and the virtual-keyboard submit trim.
#[derive(Clone, Debug, PartialEq)]
pub struct WrappedSearchBar {
    /// The inner `_searchBar` (`:24`) — scale 0.8 (`:126`), the ctor's
    /// `SetContents(null, forced: true)` shows the placeholder (`:85`).
    pub bar: SearchBar,
    /// `_theme` (`:36`).
    pub theme: SearchTheme,
    /// `_searchString` (`:32`) — the last `OnContentsChanged` payload
    /// (`:213-220`), the virtual keyboard's seed text (`:173`).
    pub search_string: String,
    /// The box panel's live `BorderColor` — the theme colour at rest,
    /// `Main.OurFavoriteColor` while taking input (`:199-211`).
    pub border_color: [u8; 4],
    /// `HideSearchButton` (`:88-93`): the magnifier removed, the box grows
    /// to `Width (-3, 1)` instead of `(-buttonW-3, 1)` (`:112`).
    pub search_button_hidden: bool,
}

impl WrappedSearchBar {
    /// The ctor (`:76-86`): 24 px tall, full width, padding 0; the empty
    /// text defaults to `UI.PlayerNameSlot` (`:80`) — the caller passes the
    /// resolved string. The inner bar's cap stays 50.
    pub fn new(empty_text: &str, theme: SearchTheme) -> Self {
        Self {
            bar: SearchBar::new(empty_text, 0.8),
            theme,
            search_string: String::new(),
            border_color: search_box_color(theme),
            search_button_hidden: false,
        }
    }

    /// The box panel's width style (`:112`, `:91`): `(0 - buttonW - 3, 1)`
    /// normally, `(-3, 1)` after `HideSearchButton`. Returns the pixel part
    /// (the percent is 1 either way); `button_w` is the magnifier's
    /// `Width.Pixels` (22, `UIImageButton.cs:31-32`).
    pub fn box_width_pixels(&self, button_w: f32) -> f32 {
        if self.search_button_hidden {
            -3.0 // `:91`
        } else {
            -button_w - 3.0 // `:112`
        }
    }

    /// `SetContents` (`:56-59`) + `UpdateSearchContents` (`:213-220`): the
    /// inner swap, then `_searchString` takes the **raw** payload (before
    /// the inner trim). Returns the inner's change-event flag.
    pub fn set_contents(&mut self, contents: &str, forced: bool) -> bool {
        let changed = self.bar.set_contents(contents, forced);
        if changed {
            self.search_string = contents.to_string(); // `:215`
        }
        changed
    }

    /// `OnStartTakingInput`/`OnEndTakingInput` (`:199-211`): the border
    /// swaps to `Main.OurFavoriteColor` (255, 231, 69) while writing, back
    /// to the theme colour after. Call after the inner bar's toggle.
    pub fn sync_focus_border(&mut self) {
        self.border_color = if self.bar.is_writing_text {
            [255, 231, 69, 255] // `:201`
        } else {
            search_box_color(self.theme) // `:206-210`
        };
    }

    /// `searchCancelButton_OnClick` (`:153-164`): with contents, clear
    /// (forced) and play sound **11**; without, just sound **12**. Returns
    /// the sound id.
    pub fn cancel_click(&mut self) -> i32 {
        if self.bar.has_contents() {
            self.set_contents("", true); // `SetContents(null, forced)` (`:157`)
            11
        } else {
            12
        }
    }

    /// `SearchBox_OnRightClick` (`:230-237`): clear (forced), then start
    /// taking input if not already. Returns `true` when the toggle fired.
    pub fn right_click(&mut self) -> bool {
        self.set_contents("", true);
        if !self.bar.is_writing_text {
            self.bar.toggle_taking_text();
            self.sync_focus_border();
            true
        } else {
            false
        }
    }

    /// `Click_SearchArea` (`:222-228`): the magnifier/panel click toggles
    /// input **unless** the event target's parent is the box panel (a
    /// click on the cancel button or the inner bar — those don't toggle).
    /// Returns `true` when the toggle fired.
    pub fn search_area_click(&mut self, target_parent_is_box: bool) -> bool {
        if !target_parent_is_box {
            self.bar.toggle_taking_text();
            self.sync_focus_border();
            true
        } else {
            false
        }
    }

    /// `Update`'s outside-click close (`:239-246`): while writing, a left
    /// or right press with **no** child hovered toggles the input off.
    /// Returns `true` when the toggle fired.
    pub fn update_outside_click(
        &mut self,
        mouse_left: bool,
        mouse_right: bool,
        any_child_hovering: bool,
    ) -> bool {
        if self.bar.is_writing_text && (mouse_left || mouse_right) && !any_child_hovering {
            self.bar.toggle_taking_text();
            self.sync_focus_border();
            true
        } else {
            false
        }
    }

    /// `SubmitVirtualText` (`:184-188`): the keyboard's text is trimmed
    /// before the contents swap; `GoBackFromVirtualKeyboard` (`:190-197`)
    /// then closes the input latch if still open.
    pub fn submit_virtual_text(&mut self, text: &str) {
        self.set_contents(text.trim(), false); // `:186`
        if self.bar.is_writing_text {
            self.bar.toggle_taking_text(); // `:192-195`
            self.sync_focus_border();
        }
    }
}

#[cfg(test)]
mod wrapped_tests {
    use super::*;

    /// The theme colours + focus border (`UIWrappedSearchBar.cs:117-123`,
    /// `:199-211`): Blue keeps (35,40,83), Red shifts through the cyan
    /// helper; writing swaps the border to OurFavoriteColor and back.
    #[test]
    fn the_wrapped_bar_swaps_the_focus_border_per_theme() {
        let mut w = WrappedSearchBar::new("Name:", SearchTheme::Blue);
        assert_eq!(w.border_color, [35, 40, 83, 255]);
        assert_eq!(w.bar.text_scale, 0.8);
        assert_eq!(w.box_width_pixels(22.0), -25.0);
        w.search_button_hidden = true;
        assert_eq!(w.box_width_pixels(22.0), -3.0);
        w.search_area_click(false);
        assert!(w.bar.is_writing_text);
        assert_eq!(w.border_color, [255, 231, 69, 255]);
        assert!(!w.search_area_click(true), "a box-child click never toggles");
        w.update_outside_click(true, false, false);
        assert!(!w.bar.is_writing_text);
        assert_eq!(w.border_color, [35, 40, 83, 255]);
        let red = WrappedSearchBar::new("", SearchTheme::Red);
        assert_eq!(
            red.border_color,
            super::super::color::shift_blue_to_cyan([35, 40, 83, 255])
        );
    }

    /// The cancel/right-click/submit arms (`:153-164`, `:230-237`,
    /// `:184-197`): cancel sounds 11 with contents / 12 without, the
    /// right-click clears + opens, the virtual submit trims and closes.
    #[test]
    fn the_cancel_and_submit_arms_match_the_sounds_and_trims() {
        let mut w = WrappedSearchBar::new("Name:", SearchTheme::Blue);
        assert_eq!(w.cancel_click(), 12, "empty: just the tick");
        w.set_contents("sword", false);
        assert_eq!(w.search_string, "sword");
        assert_eq!(w.cancel_click(), 11);
        assert!(!w.bar.has_contents());
        assert_eq!(w.bar.display_text, "Name:");
        assert!(w.right_click(), "the right-click opens the input");
        assert!(w.bar.is_writing_text);
        assert!(!w.right_click(), "already writing: no second toggle");
        w.submit_virtual_text("  axe  ");
        assert_eq!(w.bar.actual_contents, "axe", "the trim (`:186`)");
        assert!(!w.bar.is_writing_text, "the go-back closes the latch");
        assert_eq!(w.border_color, [35, 40, 83, 255]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `TrimUserString` (`Utils.cs:2839-2850`): the cut is in UTF-16 units
    /// and a dangling high surrogate backs the cut off one more — trimming
    /// an emoji at its middle drops the whole pair.
    #[test]
    fn the_trim_counts_utf16_units_and_drops_dangling_surrogates() {
        assert_eq!(trim_user_string("hello", 10), "hello");
        assert_eq!(trim_user_string("hello", 3), "hel");
        assert_eq!(trim_user_string("a😀b", 2), "a", "the split pair drops");
        assert_eq!(trim_user_string("a😀b", 3), "a😀");
        assert_eq!(trim_last_character("a😀"), "a", "one char = two units");
        assert_eq!(trim_last_character("ab"), "a");
        assert_eq!(trim_last_character(""), "");
    }

    /// `SetContents` (`UISearchBar.cs:67-89`): the same-string no-op, the
    /// Gray placeholder / White contents swap, the 50-unit cap read-back,
    /// and `HasContents`'s whitespace gate.
    #[test]
    fn set_contents_swaps_the_placeholder_and_caps_the_length() {
        let mut bar = SearchBar::new("Name:", 0.8);
        assert_eq!(bar.max_input_length, 50);
        assert_eq!(bar.display_text, "Name:");
        assert_eq!(bar.text_color(), [128, 128, 128, 255]);
        assert!(!bar.has_contents());
        assert!(bar.set_contents("sword", false));
        assert_eq!(bar.text_color(), [255, 255, 255, 255]);
        assert!(!bar.set_contents("sword", false), "same string: no event");
        assert!(bar.set_contents("sword", true), "forced fires anyway");
        assert!(bar.has_contents());
        let long = "x".repeat(60);
        assert!(bar.set_contents(&long, false));
        assert_eq!(bar.actual_contents.len(), 50, "the cap reads back (`:81`)");
        bar.set_contents("   ", false);
        assert!(!bar.has_contents(), "whitespace is not contents (`:23`)");
        bar.set_contents("", false);
        assert_eq!(bar.display_text, "Name:", "empty shows the placeholder");
    }

    /// The display trim (`:91-114`): the loop drops characters until the
    /// int-truncated text right edge fits, writing each trim back; a
    /// zero-sized element skips it entirely.
    #[test]
    fn the_display_trim_drops_until_the_int_edges_fit() {
        let mut bar = SearchBar::new("", 1.0);
        bar.set_contents("abcdef", false);
        // Element right = 100, text at x=50, 10 px per char, padding 0:
        // "abcdef" measures 60 -> right 110 > 100; trims to "abcde" (100).
        bar.trim_display_if_over_dimensions([0.0, 0.0, 100.0, 20.0], 50.0, 0, |t| {
            t.len() as f32 * 10.0
        });
        assert_eq!(bar.display_text, "abcde");
        assert_eq!(bar.actual_contents, "abcde", "the write-back (`:111`)");
        // Padding shrinks the budget: right must be <= 95 -> "abcd".
        bar.trim_display_if_over_dimensions([0.0, 0.0, 100.0, 20.0], 50.0, 5, |t| {
            t.len() as f32 * 10.0
        });
        assert_eq!(bar.display_text, "abcd");
        // A zero-sized element never trims (`:94`).
        bar.trim_display_if_over_dimensions([0.0, 0.0, 0.0, 0.0], 50.0, 0, |t| {
            t.len() as f32 * 10.0
        });
        assert_eq!(bar.display_text, "abcd");
    }

    /// The input latch (`:169-196`): toggle flips, deactivate only fires
    /// while writing.
    #[test]
    fn the_input_latch_toggles_and_deactivate_closes_it() {
        let mut bar = SearchBar::new("", 1.0);
        assert!(!bar.on_deactivate(), "idle deactivate does nothing");
        assert!(bar.toggle_taking_text());
        assert!(bar.is_writing_text);
        assert!(bar.on_deactivate());
        assert!(!bar.is_writing_text);
    }
}
