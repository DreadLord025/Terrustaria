//! The Journey powers menu's element markers. W4-d82 (research/18b row 25):
//! `UICreativePowerButton` (`GameContent/UI/Elements/UICreativePowerButton.cs:5-7`).
//!
//! The class is **empty** in 1.4.5.8 — `public class UICreativePowerButton :
//! UIElement { }` — a type marker with no fields, no methods and no overrides,
//! and a grep of the decompiled tree finds no constructor call or subclass
//! (the powers menu builds plain `UIPanel`s). The faithful port is the marker
//! itself: a unit type carrying the citation, so the census row is closed
//! without inventing behaviour the source does not have.

/// `UICreativePowerButton` (`UICreativePowerButton.cs:5-7`): an empty
/// `UIElement` subclass. No members to port; the type exists so a future
/// powers-menu port can name it 1:1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CreativePowerButton;

#[cfg(test)]
mod tests {
    use super::*;

    /// The class is a marker: the port is a unit type (no state, no behaviour).
    #[test]
    fn the_power_button_is_an_empty_marker() {
        assert_eq!(std::mem::size_of::<CreativePowerButton>(), 0);
        assert_eq!(CreativePowerButton, CreativePowerButton::default());
    }
}
