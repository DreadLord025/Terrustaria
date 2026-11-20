//! The `UIState` root element — W4-f23 (census lane 4, research/18e):
//! `Terraria/UI/UIState.cs:3-13`, the base every screen instantiates. The
//! d81-d83 element census covered `Elements/` + `UISortableElement`; the
//! screens waves (d84-d86/f22) cover the `UIState` *subclasses*; the base
//! class itself stayed research/18's PARTIAL row ("no shared root type").
//! This ports the root's two observable facts: the 100%×100% ctor rect and
//! the `NoGamepadSupport` flag's only consumer arm.
//!
//! Immediate-mode ruling (research/18): the screens keep their per-screen
//! root rects; this is the shared pure helper they can adopt, plus the
//! gamepad gate as data (gamepad navigation itself is unported — research/18
//! deviation 10, Stage-1 exclusion).
#![allow(dead_code)] // wiring lands with the screens' root-rect adoption

use super::uistate::{self, Style};

/// `UIState` (`UIState.cs:3-13`): a `UIElement` whose ctor sets
/// `Width.Precent = 1f`, `Height.Precent = 1f` and recalculates (`:9-11`),
/// plus the public `NoGamepadSupport` flag (`:5`, default false).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateRoot {
    /// `NoGamepadSupport` (`UIState.cs:5`). The only setter in 1.4.5.8 is
    /// `UIWorldGenDebug`'s ctor (`UIWorldGenDebug.cs:883`, dev-only screen).
    pub no_gamepad_support: bool,
}

/// The ctor's resolved rect (`UIState.cs:9-11` → `UIElement.Recalculate`):
/// `Left`/`Top` stay `StyleDimension.Empty` (0), the width/height are 100%
/// of the parent (`UserInterface.GetDimensions()`'s screen rect,
/// `UserInterface.cs:332-336`), alignments 0, `MaxWidth` the default `Fill`
/// (= parent width, `UIElement.cs:33-39`), so the root is exactly the parent
/// rect. `parent` is the screen rect `[x, y, w, h]`.
pub fn root_rect(parent: [f32; 4]) -> [f32; 4] {
    uistate::resolve(
        parent,
        Style::px(0.0),  // Left unset (`StyleDimension.Empty`)
        Style::px(0.0),  // Top unset
        Style::pct(1.0), // `base.Width.Precent = 1f` (`:9`)
        Style::pct(1.0), // `base.Height.Precent = 1f` (`:10`)
        0.0,             // HAlign default
        0.0,             // VAlign default
        0.0,             // MinWidth Empty
        0.0,             // MinHeight Empty
        parent[2],       // MaxWidth = Fill (parent width)
        [0.0; 4],        // no padding
    )
}

/// `UILinkPointNavigator.Update`'s `NoGamepadSupport` arm
/// (`GameContent/UI/Gamepad/UILinkPointNavigator.cs:280-283`):
/// `if (flag && Main.gameMenu && Main.MenuUI.IsVisible &&
/// Main.MenuUI.CurrentState != null && CurrentState.NoGamepadSupport)
/// flag = false;` — the menu's gamepad navigation stands down while a
/// no-gamepad state is up. The other `flag` gates (`:264-279`: rebinding
/// lock, in-game UI mode, building mode, navigator availability) are the
/// caller's; this ports only the state's arm. `state` is `None` for the
/// C#'s null `CurrentState` (the arm then never fires).
pub fn no_gamepad_blocks_navigation(
    game_menu: bool,
    menu_visible: bool,
    state: Option<&StateRoot>,
) -> bool {
    game_menu && menu_visible && state.is_some_and(|s| s.no_gamepad_support)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ctor rect (`UIState.cs:9-11`): 100%×100% of the parent, no shift.
    #[test]
    fn the_root_fills_its_parent_exactly() {
        assert_eq!(
            root_rect([0.0, 0.0, 1600.0, 900.0]),
            [0.0, 0.0, 1600.0, 900.0]
        );
        // A non-zero parent origin rides through (`UIElement.cs:308-309`).
        assert_eq!(
            root_rect([10.0, 20.0, 800.0, 600.0]),
            [10.0, 20.0, 800.0, 600.0]
        );
    }

    /// `NoGamepadSupport` defaults false (`UIState.cs:5`); the navigator arm
    /// (`UILinkPointNavigator.cs:280-283`) needs ALL of gameMenu + visible +
    /// non-null state + the flag.
    #[test]
    fn the_gamepad_gate_needs_every_conjunct() {
        let plain = StateRoot::default();
        assert!(!plain.no_gamepad_support);
        let blocked = StateRoot {
            no_gamepad_support: true,
        };
        assert!(no_gamepad_blocks_navigation(true, true, Some(&blocked)));
        assert!(!no_gamepad_blocks_navigation(false, true, Some(&blocked)));
        assert!(!no_gamepad_blocks_navigation(true, false, Some(&blocked)));
        assert!(!no_gamepad_blocks_navigation(true, true, None));
        assert!(!no_gamepad_blocks_navigation(true, true, Some(&plain)));
    }
}
