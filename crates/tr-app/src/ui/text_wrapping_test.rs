//! The text-wrap test harness's nested element — W4-f23 (census lane 4,
//! research/18e): `UITextWrappingTest.TestElement`
//! (`GameContent/UI/States/UITextWrappingTest.cs:26-108`) + the `Mode` enum
//! (`:16-24`). The screen is the d86 scope-out (dev-only, reached from debug
//! chat commands); its nested `UIElement` subclass was in no element census
//! row. This ports the element's layout machine — the wrap-width fallback,
//! the 30 px line pitch, the per-mode MinHeight source and row anchors. The
//! wrap/measure functions themselves (`Utils.WordwrapString*`,
//! `ChatManager.GetStringSize`) are the caller's seams: the port takes their
//! outputs (line counts / measured heights) as data.
//!
//! Wiring census (W4-f23 wiring pass): DEBUG-DEFERRED — the `TR_UI` preset
//! was evaluated and is NOT cheap: `UITextWrappingTest` is a whole screen
//! (`UIList` + mode-cycling buttons + the `UIText` mode label, `:110-260`),
//! so a preset needs a new `FrontScreen` variant with exhaustive
//! draw/click/key arms plus the screen port itself — d86 scope, not a wiring
//! pass. The layout machine stays parked for it.
#![allow(dead_code)] // dev-only harness (d86 scope-out); helpers parked for it

/// `UITextWrappingTest.Mode` (`:16-24`), declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    UiText,
    SignsAndNpcChat,
    WordwrapStringLegacy,
    DrawColorCodedStringWithShadow,
    DrawColorCodedStringLegacy,
    MultilineChat,
}

/// `DrawSelf`'s width read (`:46-50`): `num = GetInnerDimensions().Width`,
/// and `num <= 0f` falls back to **1000f**. The wrap calls then take
/// `(int)(num / scale)` (`:55`/`:67`/`:79`) — truncating toward zero.
/// Returns `(effective_width, wrap_max_width)`.
pub fn wrap_width(inner_width: f32, scale: f32) -> (f32, i32) {
    let num = if inner_width <= 0.0 {
        1000.0
    } else {
        inner_width
    };
    (num, (num / scale) as i32)
}

/// The wrapped modes' line pitch (`:56`/`:68`/`:80`): `30f * scale`.
pub fn line_pitch(scale: f32) -> f32 {
    30.0 * scale
}

/// Row `j`'s draw anchor Y (`:61`/`:73`/`:85`): `vector.Y + j * pitch` —
/// X stays the element position's X (`:45`).
pub fn line_y(base_y: f32, j: usize, scale: f32) -> f32 {
    base_y + j as f32 * line_pitch(scale)
}

/// `MinHeight.Set` per mode: the three wrapped modes set
/// `lineCount * 30f * scale` (`:57`/`:69`/`:81`); the two color-coded modes
/// set the measured `GetStringSize(...).Y` (`:93`/`:102`); `Mode.UIText` has
/// NO switch case (`:51-106`) — it draws nothing and never touches
/// MinHeight (the harness's `UIText` row is a plain `UIText` child instead).
/// `lines` is the wrap's line count, `measured_y` the string-size height.
pub fn min_height(mode: Mode, lines: usize, measured_y: f32, scale: f32) -> Option<f32> {
    match mode {
        Mode::SignsAndNpcChat | Mode::WordwrapStringLegacy | Mode::MultilineChat => {
            Some(lines as f32 * line_pitch(scale))
        }
        Mode::DrawColorCodedStringWithShadow | Mode::DrawColorCodedStringLegacy => Some(measured_y),
        Mode::UiText => None,
    }
}

/// What `DrawSelf` draws per mode (the draw calls are the renderer's seam;
/// this is the dispatch the C# switch performs, `:51-106`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DrawPlan {
    /// No case — nothing (`Mode.UIText`).
    Nothing,
    /// `Utils.WordwrapString` + per-line `DrawBorderStringFourWay`
    /// White-on-Black (`:55-62`).
    WrapFourWay,
    /// `Utils.WordwrapStringLegacy` + the same four-way rows (`:67-74`).
    WrapLegacyFourWay,
    /// `Utils.WordwrapStringSmart` + per-line
    /// `ChatManager.DrawColorCodedStringWithShadow` snippets (`:79-86`).
    WrapSmartShadow,
    /// One `DrawColorCodedStringWithShadow` at the position (`:91`).
    ColorCodedShadow,
    /// `DrawColorCodedStringShadow` in **Black** then
    /// `DrawColorCodedString` in White, both at the position (`:99-100`).
    ColorCodedLegacyTwoPass,
}

/// The mode → draw dispatch (`:51-106`).
pub fn draw_plan(mode: Mode) -> DrawPlan {
    match mode {
        Mode::UiText => DrawPlan::Nothing,
        Mode::SignsAndNpcChat => DrawPlan::WrapFourWay,
        Mode::WordwrapStringLegacy => DrawPlan::WrapLegacyFourWay,
        Mode::MultilineChat => DrawPlan::WrapSmartShadow,
        Mode::DrawColorCodedStringWithShadow => DrawPlan::ColorCodedShadow,
        Mode::DrawColorCodedStringLegacy => DrawPlan::ColorCodedLegacyTwoPass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The width fallback + truncation (`:46-50`, `:55`): a zero/negative
    /// inner width wraps at 1000, the max width is `(int)(num/scale)`.
    #[test]
    fn the_wrap_width_falls_back_to_1000_and_truncates() {
        assert_eq!(wrap_width(0.0, 1.0), (1000.0, 1000));
        assert_eq!(wrap_width(-5.0, 2.0), (1000.0, 500));
        assert_eq!(wrap_width(300.0, 1.0), (300.0, 300));
        // 300 / 0.7 = 428.57… → 428 (int truncation, not round).
        assert_eq!(wrap_width(300.0, 0.7), (300.0, 428));
    }

    /// The pitch + row anchors + MinHeight sources (`:56-57`, `:93`):
    /// wrapped modes ride lineCount*30*scale, color-coded modes the
    /// measured height, UIText never sets it.
    #[test]
    fn min_height_and_rows_follow_the_mode() {
        assert_eq!(line_pitch(0.5), 15.0);
        assert_eq!(line_y(100.0, 3, 0.5), 145.0);
        assert_eq!(min_height(Mode::SignsAndNpcChat, 4, 999.0, 0.5), Some(60.0));
        assert_eq!(
            min_height(Mode::WordwrapStringLegacy, 2, 999.0, 1.0),
            Some(60.0)
        );
        assert_eq!(min_height(Mode::MultilineChat, 3, 999.0, 1.0), Some(90.0));
        assert_eq!(
            min_height(Mode::DrawColorCodedStringWithShadow, 4, 42.0, 1.0),
            Some(42.0)
        );
        assert_eq!(
            min_height(Mode::DrawColorCodedStringLegacy, 4, 42.0, 1.0),
            Some(42.0)
        );
        assert_eq!(
            min_height(Mode::UiText, 4, 42.0, 1.0),
            None,
            "no switch case"
        );
    }

    /// The draw dispatch (`:51-106`): every mode's call shape, incl. the
    /// legacy two-pass Black-shadow-then-White order and the UIText no-op.
    #[test]
    fn the_draw_dispatch_matches_the_switch() {
        assert_eq!(draw_plan(Mode::UiText), DrawPlan::Nothing);
        assert_eq!(draw_plan(Mode::SignsAndNpcChat), DrawPlan::WrapFourWay);
        assert_eq!(
            draw_plan(Mode::WordwrapStringLegacy),
            DrawPlan::WrapLegacyFourWay
        );
        assert_eq!(draw_plan(Mode::MultilineChat), DrawPlan::WrapSmartShadow);
        assert_eq!(
            draw_plan(Mode::DrawColorCodedStringWithShadow),
            DrawPlan::ColorCodedShadow
        );
        assert_eq!(
            draw_plan(Mode::DrawColorCodedStringLegacy),
            DrawPlan::ColorCodedLegacyTwoPass
        );
    }
}
