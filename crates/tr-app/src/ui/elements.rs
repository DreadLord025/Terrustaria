//! The small shared `GameContent/UI/Elements` classes with no screen of their
//! own (W4-d81, research/18b census): the three marker interfaces
//! (`IColorable`, `IGroupOptionButton`, `IManuallyOrderedUIElement`), the
//! `ExtraBestiaryInfoPageInformation` carrier and `PowerStripUIElement`'s strip
//! layout. (`UICyclingImage`/`UICreativePowerButton` are d82 rows — they live
//! in `ui/images.rs`/`ui/creative_powers.rs` on that lane.)
//!
//! Immediate-mode ruling (research/18, ANALYSIS "UIElement framework"): no
//! retained tree — the classes port as pure layout/state helpers the screens
//! call per frame, exactly like `world_creation`'s `GroupOptionButton` rows.
//! Gamepad snap points (`SetSnapPoint`) stay unported with the rest of the
//! gamepad navigation (research/18 framework gap).
#![allow(dead_code)] // wiring lands with the creative/bestiary screens (W4-d81..d83)

use super::color;
use super::uistate;

// --- IColorable / IGroupOptionButton / IManuallyOrderedUIElement --------------

/// `IColorable` (`IColorable.cs:5-8`): a `Color { get; set; }` property. The
/// C#'s implementors are `UIImageFramed` and the info-page lines; the port's
/// draw helpers take the tint as an argument, so the trait marks the state
/// carriers that keep one.
pub trait Colorable {
    /// The `Color` getter.
    fn colorable_color(&self) -> [u8; 4];
    /// The `Color` setter.
    fn set_colorable_color(&mut self, color: [u8; 4]);
}

/// `IGroupOptionButton` (`IGroupOptionButton.cs:5-10`): the two colour setters
/// `UIBestiaryFilteringOptionsGrid`/`SortingOptionsGrid` drive through the
/// interface. [`GroupOptionButtonState`] is the port's implementor.
pub trait GroupOptionButton {
    /// `SetColorsBasedOnSelectionState` (`GroupOptionButton.cs:277-284`): the
    /// picked/unpicked override pair and their opacities.
    fn set_colors_based_on_selection_state(
        &mut self,
        picked_color: [u8; 4],
        unpicked_color: [u8; 4],
        opacity_picked: f32,
        opacity_not_picked: f32,
    );
    /// `SetBorderColor` (`GroupOptionButton.cs:286-289`).
    fn set_border_color(&mut self, color: [u8; 4]);
}

/// `IManuallyOrderedUIElement` (`IManuallyOrderedUIElement.cs:3-6`): the
/// `OrderInUIList` sort key the bestiary info lines carry.
pub trait ManuallyOrdered {
    /// The `OrderInUIList` getter.
    fn order_in_ui_list(&self) -> i32;
    /// The `OrderInUIList` setter.
    fn set_order_in_ui_list(&mut self, order: i32);
}

/// `UIBestiaryInfoLine.CompareTo` (`UIBestiaryInfoLine.cs:92-99`): two manually
/// ordered elements compare by `OrderInUIList` (`int.CompareTo` — the sign of
/// the difference, not the difference itself).
pub fn compare_manual_order(a: i32, b: i32) -> std::cmp::Ordering {
    a.cmp(&b)
}

// --- GroupOptionButton's colour state ----------------------------------------

/// The `GroupOptionButton<T>` colour fields (`GroupOptionButton.cs:26-52`) the
/// bestiary grids mutate through `SetColor`/`SetColorsBasedOnSelectionState`.
/// `world_creation::option_button_colors` ports the *default* chain for the
/// creation screen; this struct carries the mutable state the grids override.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupOptionButtonState {
    /// `_color` — ctor default `Colors.InventoryDefaultColor` (`:133`).
    pub color: [u8; 4],
    /// `_opacity = 0.7f` (`:36`).
    pub opacity: f32,
    /// `_borderColor = Color.White` (`:120`).
    pub border_color: [u8; 4],
    /// `FadeFromBlack = 1f` (`:30`).
    pub fade_from_black: f32,
    /// `InnerHighlightRim = 7` (`:32`) — the selected rim's inset.
    pub inner_highlight_rim: f32,
    /// `_whiteLerp = 0.7f` (`:34`).
    pub white_lerp: f32,
    /// `_UseOverrideColors` (`:44`) — set by `SetColorsBasedOnSelectionState`.
    pub use_override: bool,
    /// `_overridePickedColor` / `_overrideUnpickedColor` (`:46-48`).
    pub override_picked: [u8; 4],
    pub override_unpicked: [u8; 4],
    /// `_overrideOpacityPicked` / `_overrideOpacityUnpicked` (`:50-52`).
    pub override_opacity_picked: f32,
    pub override_opacity_unpicked: f32,
}

impl Default for GroupOptionButtonState {
    fn default() -> Self {
        Self {
            color: uistate::INVENTORY_DEFAULT_COLOR, // `:133`
            opacity: 0.7,                            // `:36`
            border_color: [255, 255, 255, 255],      // `:120`
            fade_from_black: 1.0,                    // `:30`
            inner_highlight_rim: 7.0,                // `:32`
            white_lerp: 0.7,                         // `:34`
            use_override: false,
            override_picked: [255, 255, 255, 255], // `:46-48`
            override_unpicked: [255, 255, 255, 255],
            override_opacity_picked: 0.0,
            override_opacity_unpicked: 0.0,
        }
    }
}

impl GroupOptionButton for GroupOptionButtonState {
    fn set_colors_based_on_selection_state(
        &mut self,
        picked_color: [u8; 4],
        unpicked_color: [u8; 4],
        opacity_picked: f32,
        opacity_not_picked: f32,
    ) {
        // `GroupOptionButton.cs:277-284`.
        self.use_override = true;
        self.override_picked = picked_color;
        self.override_unpicked = unpicked_color;
        self.override_opacity_picked = opacity_picked;
        self.override_opacity_unpicked = opacity_not_picked;
    }

    fn set_border_color(&mut self, color: [u8; 4]) {
        self.border_color = color; // `:286-289`
    }
}

impl GroupOptionButtonState {
    /// `SetColor` (`GroupOptionButton.cs:271-275`).
    pub fn set_color(&mut self, color: [u8; 4], opacity: f32) {
        self.color = color;
        self.opacity = opacity;
    }

    /// `DrawSelf`'s colour pick (`GroupOptionButton.cs:225-232`): the override
    /// pair when `_UseOverrideColors`, the base `_color`/`_opacity` otherwise.
    pub fn effective(&self, selected: bool) -> ([u8; 4], f32) {
        if self.use_override {
            if selected {
                (self.override_picked, self.override_opacity_picked)
            } else {
                (self.override_unpicked, self.override_opacity_unpicked)
            }
        } else {
            (self.color, self.opacity)
        }
    }

    /// The `PanelGrayscale` fill (`GroupOptionButton.cs:233`):
    /// `Color.Lerp(Color.Black, color, FadeFromBlack) * opacity`.
    pub fn fill_color(&self, selected: bool) -> [u8; 4] {
        let (c, opacity) = self.effective(selected);
        let lerped = super::char_creation::lerp_rgb([0, 0, 0, 255], c, self.fade_from_black);
        color::mul(lerped, opacity)
    }

    /// The selected rim / idle icon tint (`GroupOptionButton.cs:236`/`:247`):
    /// `Color.Lerp(color, White, _whiteLerp) * opacity`.
    pub fn rim_color(&self, selected: bool) -> [u8; 4] {
        let (c, opacity) = self.effective(selected);
        let lerped = super::char_creation::lerp_rgb(c, [255, 255, 255, 255], self.white_lerp);
        color::mul(lerped, opacity)
    }

    /// The hovered border's colour (`GroupOptionButton.cs:238-241`): the
    /// plain `_borderColor`, no opacity multiply — the border draws over the
    /// rim when both states hold.
    pub fn hover_border_color(&self) -> [u8; 4] {
        self.border_color
    }

    /// The icon tint (`GroupOptionButton.cs:242-249`): `IconColor` while
    /// hovered or selected (`:244-245`'s gate inverted), else
    /// `Color.Lerp(color, IconColor, _whiteLerp) * opacity` (`:247`).
    pub fn icon_tint(&self, selected: bool, hovered: bool, icon_color: [u8; 4]) -> [u8; 4] {
        if hovered || selected {
            return icon_color;
        }
        let (c, opacity) = self.effective(selected);
        let lerped = super::char_creation::lerp_rgb(c, icon_color, self.white_lerp);
        color::mul(lerped, opacity)
    }

    /// The selected rim's inset rect (`GroupOptionButton.cs:236`):
    /// `X/Y + InnerHighlightRim`, `Width/Height - InnerHighlightRim * 2`.
    pub fn highlight_rect(&self, dims: [f32; 4]) -> [f32; 4] {
        let rim = self.inner_highlight_rim;
        [
            dims[0] + rim,
            dims[1] + rim,
            dims[2] - rim * 2.0,
            dims[3] - rim * 2.0,
        ]
    }

    /// The `DrawSelf` hover-sound latch (`GroupOptionButton.cs:212-223`):
    /// sound 12 fires once on the hovered edge (`!_soundedHover`), the latch
    /// holds while hovered and clears on mouse-out. Returns whether to play.
    /// (The same one-shot d82's `HoverSoundLatch` ports for the difficulty/
    /// hair buttons — the merge owner may fold the two into one helper.)
    pub fn hover_sound_tick(hovered: bool, sounded_hover: &mut bool) -> bool {
        if hovered {
            let play = !*sounded_hover; // `:214-217`
            *sounded_hover = true; // `:218`
            play
        } else {
            *sounded_hover = false; // `:222`
            false
        }
    }
}

/// The ctor's element size (`GroupOptionButton.cs:124-125`):
/// `Width 44`, `Height 34`.
#[allow(dead_code)] // wiring lands with the bestiary grids (W4-d81)
pub const GROUP_OPTION_SIZE: [f32; 2] = [44.0, 34.0];

/// The title `UIText`'s rect inside the button (`GroupOptionButton.cs:136-142`
/// / `SetText` `:155-161`): `Width (-title_width_reduction, 1)`, `HAlign
/// title_alignment_x`, `VAlign 0.5`, `Top 0` — the text box is the button
/// minus the reduction, centred vertically; the ctor defaults are
/// `titleAlignmentX = 0.5`, `titleWidthReduction = 10` (`:118`).
#[allow(dead_code)] // wiring lands with the bestiary grids (W4-d81)
pub fn group_option_title_rect(
    dims: [f32; 4],
    title_height: f32,
    title_alignment_x: f32,
    title_width_reduction: f32,
) -> [f32; 4] {
    let w = dims[2] - title_width_reduction;
    [
        dims[0] + (dims[2] - w) * title_alignment_x,
        dims[1] + (dims[3] - title_height) * 0.5,
        w,
        title_height,
    ]
}

// --- ExtraBestiaryInfoPageInformation -----------------------------------------

/// `BestiaryUnlockProgressReport` (`BestiaryUnlockProgressReport.cs:3-19`):
/// the roster totals the info page's completion bar reads.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BestiaryUnlockProgressReport {
    /// `EntriesTotal` (`:5`).
    pub entries_total: i32,
    /// `CompletionAmountTotal` (`:7`).
    pub completion_amount_total: f32,
}

impl BestiaryUnlockProgressReport {
    /// `CompletionPercent` (`:9-18`): 1 for an empty roster, else the share.
    /// The runtime twin is `crate::bestiary::completion_percent` (the kill
    /// tracker's known-entry share, `UIBestiaryTest.GetUnlockProgress`).
    pub fn completion_percent(&self) -> f32 {
        if self.entries_total == 0 {
            return 1.0;
        }
        self.completion_amount_total / self.entries_total as f32
    }
}

/// `ExtraBestiaryInfoPageInformation` (`ExtraBestiaryInfoPageInformation.cs:5-8`):
/// the one-field carrier the bestiary info elements receive.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ExtraBestiaryInfoPageInformation {
    /// `BestiaryProgressReport` (`:7`).
    pub bestiary_progress_report: BestiaryUnlockProgressReport,
}

// --- PowerStripUIElement -------------------------------------------------------

/// `PowerStripUIElement`'s ctor constants (`PowerStripUIElement.cs:18-21`):
/// `num = 4` spacing, `num2 = num3 = 40` button size, `num4 = 44` pitch.
pub const POWER_STRIP_SPACING: f32 = 4.0;
pub const POWER_STRIP_BUTTON: f32 = 40.0;
pub const POWER_STRIP_PITCH: f32 = 44.0;

/// The strip panel's size (`PowerStripUIElement.cs:22-26`):
/// `Width = 40 + 4*2 = 48`, `Height = 40*count + 4*(1 + count)`.
pub fn power_strip_panel_size(count: usize) -> [f32; 2] {
    let n = count as f32;
    [
        POWER_STRIP_BUTTON + POWER_STRIP_SPACING * 2.0,
        POWER_STRIP_BUTTON * n + POWER_STRIP_SPACING * (1.0 + n),
    ]
}

/// Button `i`'s rect inside the strip panel at `panel` (`:36-38`):
/// `HAlign 0.5` centres the button on the 48 px panel, `Top = 4 + 44*i`.
/// The panel's `SetPadding(0)` (`:32`) keeps the inner rect equal to the outer.
pub fn power_strip_button_rect(panel: [f32; 4], button_w: f32, i: usize) -> [f32; 4] {
    [
        panel[0] + (panel[2] - button_w) * 0.5,
        panel[1] + POWER_STRIP_SPACING + POWER_STRIP_PITCH * i as f32,
        button_w,
        POWER_STRIP_BUTTON,
    ]
}

/// The strip panel's colours (`PowerStripUIElement.cs:30-31`):
/// border `(89, 116, 213) * 0.9`, background `(73, 94, 171) * 0.9`.
pub fn power_strip_colors() -> ([u8; 4], [u8; 4]) {
    (
        color::mul([89, 116, 213, 255], 0.9),
        color::mul([73, 94, 171, 255], 0.9),
    )
}

// UICyclingImage (census row 26) and UICreativePowerButton (row 25) were
// ported here in batch 1 before the 18b split was published; they belong to
// the d82 band and landed there independently (`ui/images.rs::CyclingImage`,
// `ui/creative_powers.rs::CreativePowerButton`, W4-d82 batch 1). Removed here
// per the keep-one ruling — the d82 copies are the survivors (18b census).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_report_percent_is_one_for_an_empty_roster() {
        // `BestiaryUnlockProgressReport.cs:11-14`.
        let empty = BestiaryUnlockProgressReport::default();
        assert_eq!(empty.completion_percent(), 1.0);
        let half = BestiaryUnlockProgressReport {
            entries_total: 200,
            completion_amount_total: 50.0,
        };
        assert_eq!(half.completion_percent(), 0.25);
    }

    #[test]
    fn the_power_strip_sizes_match_the_ctor() {
        // `PowerStripUIElement.cs:24-25` with 3 buttons: W 48, H 40*3+4*4 = 136.
        assert_eq!(power_strip_panel_size(3), [48.0, 136.0]);
        assert_eq!(power_strip_panel_size(0), [48.0, 4.0]);
        // Button 1 sits at Top 4 + 44 = 48, centred on the 48 px panel.
        let panel = [100.0, 50.0, 48.0, 136.0];
        assert_eq!(power_strip_button_rect(panel, 40.0, 1), [104.0, 98.0, 40.0, 40.0]);
    }

    #[test]
    fn the_power_strip_colours_truncate_like_xna() {
        // `(89,116,213)*0.9 = (80,104,191,229)`, `(73,94,171)*0.9 = (65,84,153,229)`.
        let (border, bg) = power_strip_colors();
        assert_eq!(border, [80, 104, 191, 229]);
        assert_eq!(bg, [65, 84, 153, 229]);
    }

    #[test]
    fn the_option_state_override_replaces_the_base_pair() {
        // `GroupOptionButton.cs:225-232`: the override pair wins when set.
        let mut s = GroupOptionButtonState::default();
        assert_eq!(s.effective(true), (uistate::INVENTORY_DEFAULT_COLOR, 0.7));
        s.set_colors_based_on_selection_state([152, 175, 235, 255], [63, 65, 151, 255], 1.0, 0.7);
        assert_eq!(s.effective(true), ([152, 175, 235, 255], 1.0));
        assert_eq!(s.effective(false), ([63, 65, 151, 255], 0.7));
        // `SetColor` (`:271-275`) still writes the base fields.
        s.set_color([10, 20, 30, 255], 0.5);
        assert_eq!(s.color, [10, 20, 30, 255]);
        assert_eq!(s.opacity, 0.5);
        // The override stays in force (`_UseOverrideColors` is never cleared).
        assert_eq!(s.effective(true).0, [152, 175, 235, 255]);
    }

    #[test]
    fn the_fill_and_rim_follow_the_lerp_chain() {
        // Default state, FadeFromBlack 1: fill = color * 0.7 (`:233`).
        let s = GroupOptionButtonState::default();
        assert_eq!(s.fill_color(false), color::mul(uistate::INVENTORY_DEFAULT_COLOR, 0.7));
        // The rim lerps 0.7 toward white first (`:236`): (63,65,151) ->
        // (197,198,223) truncated, then * 0.7 -> (137,138,156,178).
        let rim = s.rim_color(false);
        assert_eq!(rim, color::mul(super::super::char_creation::lerp_rgb(
            uistate::INVENTORY_DEFAULT_COLOR,
            [255, 255, 255, 255],
            0.7,
        ), 0.7));
        // FadeFromBlack 0.5 halves the fill toward black (`:233`).
        let mut faded = s;
        faded.fade_from_black = 0.5;
        let expected = color::mul(
            super::super::char_creation::lerp_rgb([0, 0, 0, 255], uistate::INVENTORY_DEFAULT_COLOR, 0.5),
            0.7,
        );
        assert_eq!(faded.fill_color(false), expected);
    }

    #[test]
    fn manual_order_compares_by_the_key() {
        // `UIBestiaryInfoLine.cs:96`: `OrderInUIList.CompareTo`.
        assert_eq!(compare_manual_order(1, 2), std::cmp::Ordering::Less);
        assert_eq!(compare_manual_order(2, 2), std::cmp::Ordering::Equal);
        assert_eq!(compare_manual_order(3, 2), std::cmp::Ordering::Greater);
    }

    #[test]
    fn the_hover_sound_latch_fires_once_per_hover() {
        // `GroupOptionButton.cs:212-223`: sound 12 on the hovered edge only.
        let mut sounded = false;
        assert!(GroupOptionButtonState::hover_sound_tick(true, &mut sounded));
        assert!(!GroupOptionButtonState::hover_sound_tick(
            true,
            &mut sounded
        ));
        // Mouse-out clears the latch (`:222`), the next hover fires again.
        assert!(!GroupOptionButtonState::hover_sound_tick(
            false,
            &mut sounded
        ));
        assert!(GroupOptionButtonState::hover_sound_tick(true, &mut sounded));
    }

    #[test]
    fn the_icon_tint_gates_on_hover_or_selection() {
        // `GroupOptionButton.cs:242-249`: IconColor raw while hovered or
        // selected; the idle tint is Lerp(color, IconColor, 0.7) * 0.7.
        let s = GroupOptionButtonState::default();
        let icon = [200, 100, 50, 255];
        assert_eq!(s.icon_tint(true, false, icon), icon);
        assert_eq!(s.icon_tint(false, true, icon), icon);
        let idle = s.icon_tint(false, false, icon);
        assert_eq!(
            idle,
            color::mul(
                super::super::char_creation::lerp_rgb(uistate::INVENTORY_DEFAULT_COLOR, icon, 0.7),
                0.7
            )
        );
        // The override pair feeds the idle tint too (`:228-232`).
        let mut o = s;
        o.set_colors_based_on_selection_state([10, 20, 30, 255], [40, 50, 60, 255], 1.0, 0.5);
        assert_eq!(
            o.icon_tint(false, false, icon),
            color::mul(
                super::super::char_creation::lerp_rgb([40, 50, 60, 255], icon, 0.7),
                0.5
            )
        );
    }

    #[test]
    fn the_highlight_rect_insets_by_the_rim_and_the_border_is_raw() {
        // `:236`: +7 inset, -14 size; `:240`: the plain `_borderColor`.
        let s = GroupOptionButtonState::default();
        assert_eq!(
            s.highlight_rect([100.0, 50.0, 44.0, 34.0]),
            [107.0, 57.0, 30.0, 20.0]
        );
        assert_eq!(s.hover_border_color(), [255, 255, 255, 255]);
        let mut b = s;
        b.set_border_color([1, 2, 3, 4]);
        assert_eq!(b.hover_border_color(), [1, 2, 3, 4]);
    }

    #[test]
    fn the_ctor_size_and_title_box_match_the_source() {
        // `:124-125`: 44x34. `:136-142`: Width (-10, 1), HAlign 0.5, VAlign 0.5.
        assert_eq!(GROUP_OPTION_SIZE, [44.0, 34.0]);
        let dims = [100.0, 50.0, 44.0, 34.0];
        assert_eq!(
            group_option_title_rect(dims, 16.0, 0.5, 10.0),
            [105.0, 59.0, 34.0, 16.0]
        );
        // The non-default alignment (`:118`'s titleAlignmentX parameter).
        assert_eq!(
            group_option_title_rect(dims, 16.0, 0.0, 10.0),
            [100.0, 59.0, 34.0, 16.0]
        );
    }
}
