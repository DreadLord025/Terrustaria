//! The chrome *element* halves the immediate ports left out — W4-d82
//! (research/18b rows 33/34): `UIHorizontalSeparator`
//! (`GameContent/UI/Elements/UIHorizontalSeparator.cs`) and `UIHeader`
//! (`UIHeader.cs`). The draw halves are landed (`panel.rs::draw_horizontal_separator`,
//! `world_load.rs::draw_header`); this module carries the constructor's texture
//! pick + size, the hit-test override, and the header's `Text` setter sizing.

/// `UIHorizontalSeparator`'s constructor (`UIHorizontalSeparator.cs:16-29`):
/// `highlightSideUp` picks `Separator1` (true, the default) or `Separator2`
/// (false); the element's size *is* the sheet's size (`Width.Set(tex.Width, 0)`,
/// `Height.Set(tex.Height, 0)`), and `Color` starts White.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeparatorSheet {
    /// `Images/UI/CharCreation/Separator1` (`:21`) — `lookups.ui_separator`.
    HighlightUp,
    /// `Images/UI/CharCreation/Separator2` (`:25`) — the flipped highlight.
    HighlightDown,
}

/// The ctor's asset pick (`:19-26`) as the branch it is.
pub fn separator_sheet(highlight_side_up: bool) -> SeparatorSheet {
    if highlight_side_up {
        SeparatorSheet::HighlightUp
    } else {
        SeparatorSheet::HighlightDown
    }
}

/// The ctor's size (`:27-28`): `(sheet_w, sheet_h)` in pixels, percent 0.
pub fn separator_size(sheet_w: f32, sheet_h: f32) -> [f32; 2] {
    [sheet_w, sheet_h]
}

/// `UIHorizontalSeparator.ContainsPoint` (`:37-40`) — **always false**: the
/// separator is drawn but never hit, whatever the rect says. The port's hit
/// tables model this by never registering the separator as an element
/// (`framework::Element` with `ignores_mouse` is the equivalent flag).
pub fn separator_contains_point() -> bool {
    false
}

/// `UIHeader.Text` setter (`UIHeader.cs:12-34`): only a *changed* string
/// re-measures. The measure (`FontAssets.DeathText.MeasureString`, `:25-27`)
/// pins `Width.Pixels`/`Height.Pixels` and zeroes both percents (`:29-30`),
/// then `Recalculate`. The port's screens measure through the font seam; this
/// carries the change gate + the percent reset the inline draw does not model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeaderText {
    text: String,
    /// `(Width.Pixels, Height.Pixels)` after the last measure.
    pub size_px: [f32; 2],
}

impl HeaderText {
    /// The setter (`:18-32`). `measure` stands in for
    /// `DeathText.Value.MeasureString` (the `!Main.dedServ` arm — the port is
    /// always a client). Returns `true` when the text changed (the C#
    /// `Recalculate` trigger); the size is the measure's output.
    pub fn set_text(&mut self, value: &str, measure: impl FnOnce(&str) -> [f32; 2]) -> bool {
        if self.text == value {
            return false; // `:20` — the no-change early out
        }
        self.text = value.to_string();
        self.size_px = measure(&self.text); // `:25-27`; percents zeroed (`:29-30`)
        true
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

/// `UIVerticalSeparator` (`GameContent/UI/Elements/UIVerticalSeparator.cs:8-34`),
/// research/18b row 56 (W4-d83): despite the name, **not** a rotated
/// `Utils.DrawPanel` — a plain `OnePixel` sheet stretched to the element
/// rect (`:24-28`, `dimensions.ToRectangle()`), White by default, with the
/// same never-hit override as the horizontal one (`:30-33`). `EdgeWidth`
/// (`:14`) is a public field the class itself never reads — dead, kept for
/// the API's sake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VerticalSeparator {
    /// `Color` (`:12`), White from the ctor (`:18`).
    pub color: [u8; 4],
    /// `EdgeWidth` (`:14`) — written by callers, never read by the draw.
    pub edge_width: i32,
    /// `Width`/`Height` — the `OnePixel` sheet's size (`:20-21`), 1×1.
    pub size: [f32; 2],
}

impl VerticalSeparator {
    /// The ctor (`:16-22`): White, size = the sheet (1×1 for `OnePixel`).
    pub fn new(sheet_size: [f32; 2]) -> Self {
        Self {
            color: [255, 255, 255, 255],
            edge_width: 0,
            size: sheet_size,
        }
    }

    /// `DrawSelf` (`:24-28`): the whole sheet stretched to
    /// `ToRectangle()` — every component int-truncated.
    pub fn draw_rect(&self, rect: [f32; 4]) -> [i32; 4] {
        [rect[0] as i32, rect[1] as i32, rect[2] as i32, rect[3] as i32]
    }

    /// `ContainsPoint` (`:30-33`) — always false, like the horizontal one.
    pub fn contains_point(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `UIVerticalSeparator` (`UIVerticalSeparator.cs:16-33`): White 1×1
    /// ctor, the stretch truncates every component, never hit.
    #[test]
    fn the_vertical_separator_stretches_one_pixel_and_never_hits() {
        let mut s = VerticalSeparator::new([1.0, 1.0]);
        assert_eq!(s.color, [255, 255, 255, 255]);
        assert_eq!(s.size, [1.0, 1.0]);
        assert_eq!(s.draw_rect([10.9, 20.8, 2.7, 100.2]), [10, 20, 2, 100]);
        assert!(!s.contains_point());
        s.edge_width = 4; // dead field: the draw ignores it (`:14` vs `:27`)
        assert_eq!(s.draw_rect([0.0, 0.0, 2.0, 50.0]), [0, 0, 2, 50]);
    }

    /// The ctor pick + size (`UIHorizontalSeparator.cs:16-29`).
    #[test]
    fn the_separator_picks_its_sheet_and_takes_its_size() {
        assert_eq!(separator_sheet(true), SeparatorSheet::HighlightUp);
        assert_eq!(separator_sheet(false), SeparatorSheet::HighlightDown);
        assert_eq!(separator_size(4.0, 4.0), [4.0, 4.0]);
        // The hit override is unconditional (`:37-40`).
        assert!(!separator_contains_point());
    }

    /// The setter's change gate (`UIHeader.cs:20`): an identical string does
    /// not re-measure; a new one pins the pixel size.
    #[test]
    fn the_header_text_setter_measures_only_on_change() {
        let mut h = HeaderText::default();
        let mut calls = 0;
        assert!(h.set_text("Generating", |t| {
            calls += 1;
            [t.len() as f32 * 10.0, 30.0]
        }));
        assert_eq!(h.size_px, [100.0, 30.0]);
        assert!(!h.set_text("Generating", |_| {
            calls += 1;
            [0.0, 0.0]
        }));
        assert_eq!(calls, 1, "the unchanged text never re-measures");
        assert_eq!(h.size_px, [100.0, 30.0]);
        assert!(h.set_text("", |_| [0.0, 30.0]));
        assert_eq!(h.text(), "");
    }
}
