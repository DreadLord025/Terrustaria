//! The grayscale text-panel button variants — W4-d83 (research/18b rows
//! 48/50): `UIResourcePackInfoButton<T>`
//! (`GameContent/UI/Elements/UIResourcePackInfoButton.cs:9-51`) and
//! `UISelectableTextPanel<T>` (`UISelectableTextPanel.cs:9-56`). Both are
//! `UITextPanel<T>` subclasses that replace the 9-slice `UIPanel` draw with
//! a spliced `PanelGrayscale` at `Lerp(Black, _color, 0.8) * 0.5` and add a
//! `CategoryPanelBorder` hover overlay; the selectable one inserts a second
//! inset splice while selected. The panel pieces ride
//! `uistate::spliced_rects`; the text draw is the base's (`DrawText`).

use super::char_creation::lerp_rgb;
use super::color;

/// The shared base splice tint (`UIResourcePackInfoButton.cs:43`,
/// `UISelectableTextPanel.cs:44`): `Color.Lerp(Color.Black, _color, 0.8f) *
/// 0.5f` — `_color` is the `UITextPanel.TextColor` (White by default,
/// `UITextPanel.cs:18`).
pub fn base_panel_tint(text_color: [u8; 4]) -> [u8; 4] {
    color::mul(lerp_rgb([0, 0, 0, 255], text_color, 0.8), 0.5)
}

/// `UIResourcePackInfoButton<T>` (`UIResourcePackInfoButton.cs:9-51`),
/// research/18b row 48 (W4-d83): the pack card's info button. The whole
/// override is the panel swap — 10/10/10/10 splices, the hover border in
/// plain White — plus the base `DrawText`; the bound `ResourcePack` is the
/// caller's data.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackInfoButton {
    /// `_color` / `TextColor` (`UITextPanel.cs:18`, `:68-77`), White default.
    pub text_color: [u8; 4],
    /// `_drawPanel` (`UITextPanel.cs:20`) — false skips both splices.
    pub draw_panel: bool,
}

impl Default for PackInfoButton {
    fn default() -> Self {
        Self {
            text_color: [255, 255, 255, 255],
            draw_panel: true,
        }
    }
}

/// The info button's draw plan (`UIResourcePackInfoButton.cs:36-50`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PackInfoDraw {
    /// The base splice: `(int)X, (int)Y, (int)W, (int)H`, 10/10/10/10
    /// (`:41-43`), tint [`base_panel_tint`]. `None` when `_drawPanel` false.
    pub base: Option<([i32; 4], [u8; 4])>,
    /// The hover border splice (`:44-47`): same rect + margins,
    /// `CategoryPanelBorder` in `Color.White`. `None` when not hovering.
    pub hover_border: Option<[i32; 4]>,
}

impl PackInfoButton {
    /// The `DrawSelf` plan; `rect` is `GetDimensions()`.
    pub fn draw_plan(&self, rect: [f32; 4], hovering: bool) -> PackInfoDraw {
        if !self.draw_panel {
            return PackInfoDraw {
                base: None,
                hover_border: None,
            };
        }
        let r = [
            rect[0] as i32,
            rect[1] as i32,
            rect[2] as i32,
            rect[3] as i32,
        ]; // `:43` int casts
        PackInfoDraw {
            base: Some((r, base_panel_tint(self.text_color))),
            hover_border: if hovering { Some(r) } else { None },
        }
    }
}

/// `UISelectableTextPanel<T>` (`UISelectableTextPanel.cs:9-56`), research/18b
/// row 50 (W4-d83): the world-creation option button. Same base splice, but
/// the selection draws a **second** splice inset 4 px on every side
/// (`num = 4`, `:41`) at `Lerp(_color, White, 0.7) * 0.5`, under the hover
/// border. The `IsSelected` `Func` is the caller's predicate — the port
/// takes the evaluated bool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectableTextPanel {
    /// `_color` / `TextColor`, White default.
    pub text_color: [u8; 4],
    /// `_drawPanel`.
    pub draw_panel: bool,
}

impl Default for SelectableTextPanel {
    fn default() -> Self {
        Self {
            text_color: [255, 255, 255, 255],
            draw_panel: true,
        }
    }
}

/// The selectable panel's draw plan (`UISelectableTextPanel.cs:36-55`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectableDraw {
    /// The base splice, 10/10/10/10 (`:44`; the local `num = 4` is only the
    /// inset, the margins stay `num2`/`num3` = 10).
    pub base: Option<([i32; 4], [u8; 4])>,
    /// The selected inset splice (`:45-48`): `(X+4, Y+4, W-8, H-8)` — the
    /// casts apply to X/Y/W/H **before** the ±4 arithmetic — tinted
    /// `Lerp(_color, White, 0.7) * 0.5`.
    pub selected: Option<([i32; 4], [u8; 4])>,
    /// The hover border (`:49-52`), White, full rect.
    pub hover_border: Option<[i32; 4]>,
}

impl SelectableTextPanel {
    /// The `DrawSelf` plan. `selected` is the evaluated `IsSelected(this)`
    /// (`:45` — a null `Func` reads as not selected).
    pub fn draw_plan(&self, rect: [f32; 4], hovering: bool, selected: bool) -> SelectableDraw {
        if !self.draw_panel {
            return SelectableDraw {
                base: None,
                selected: None,
                hover_border: None,
            };
        }
        let r = [
            rect[0] as i32,
            rect[1] as i32,
            rect[2] as i32,
            rect[3] as i32,
        ];
        let sel = if selected {
            let num = 4; // `:41`
            Some((
                [r[0] + num, r[1] + num, r[2] - num * 2, r[3] - num * 2], // `:47`
                color::mul(
                    lerp_rgb(self.text_color, [255, 255, 255, 255], 0.7),
                    0.5,
                ),
            ))
        } else {
            None
        };
        SelectableDraw {
            base: Some((r, base_panel_tint(self.text_color))),
            selected: sel,
            hover_border: if hovering { Some(r) } else { None },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shared tint (`UIResourcePackInfoButton.cs:43`): White text →
    /// Lerp(Black, White, 0.8) = (204,204,204,255), *0.5 truncates to
    /// (102,102,102,127) — the alpha halves too.
    #[test]
    fn the_grayscale_splice_tint_halves_the_faded_colour() {
        assert_eq!(base_panel_tint([255, 255, 255, 255]), [102, 102, 102, 127]);
        let b = PackInfoButton::default();
        let d = b.draw_plan([10.9, 20.8, 100.7, 30.2], false);
        assert_eq!(d.base, Some(([10, 20, 100, 30], [102, 102, 102, 127])));
        assert_eq!(d.hover_border, None);
        let h = b.draw_plan([10.9, 20.8, 100.7, 30.2], true);
        assert_eq!(h.hover_border, Some([10, 20, 100, 30]));
        let mut off = b;
        off.draw_panel = false;
        assert_eq!(off.draw_plan([0.0; 4], true).base, None);
    }

    /// The selection inset (`UISelectableTextPanel.cs:45-48`): +4/-8 on the
    /// truncated rect, tint Lerp(White, White, 0.7)*0.5 = (127,127,127,127)
    /// — 255*0.5 truncates to 127, not 128; selection and hover stack.
    #[test]
    fn the_selected_splice_insets_four_and_stacks_with_hover() {
        let p = SelectableTextPanel::default();
        let d = p.draw_plan([10.9, 20.8, 100.7, 30.2], true, true);
        assert_eq!(d.base, Some(([10, 20, 100, 30], [102, 102, 102, 127])));
        assert_eq!(d.selected, Some(([14, 24, 92, 22], [127, 127, 127, 127])));
        assert_eq!(d.hover_border, Some([10, 20, 100, 30]));
        let idle = p.draw_plan([0.0, 0.0, 100.0, 30.0], false, false);
        assert_eq!(idle.selected, None);
        assert_eq!(idle.hover_border, None);
    }
}
