//! The element identities the two select screens' hit order needs — the C#'s
//! `UIElement` references that `UserInterface.InputPointerCache` holds
//! (`UserInterface.cs:19`, `:21`), without a retained tree.
//!
//! A card's `OnLeftDoubleClick` → `PlayGame` (`UICharacterListItem.cs:68-69`,
//! `UIWorldListItem.cs:69`/`:99`) is the only double-click user in 1.4.5.8, and it
//! needs the *same* element hit twice inside 500 ms — which a per-frame
//! recompute cannot answer, hence the ids. The hit order itself is
//! `super::framework::get_element_at` (`UIElement.cs:324-352`).

/// The `New` fat button (`UICharacterSelect.cs:76-84`), the screen's last
/// appended child — so it wins a click from anything under it.
pub const NEW: u32 = 1;
/// The `Back` fat button (`:66-74`), appended just before New.
pub const BACK: u32 = 2;
/// The first card id; card `i`'s body and its four buttons occupy
/// `CARD_BASE + i*PARTS .. CARD_BASE + i*PARTS + PARTS`.
pub const CARD_BASE: u32 = 3;
/// The body plus the four `UICharacterListItem` buttons.
pub const PARTS: u32 = 5;
/// The card `UIPanel` itself.
pub const PART_BODY: u32 = 0;
/// `ButtonPlay`.
pub const PART_PLAY: u32 = 1;
/// `ButtonFavorite*`.
pub const PART_FAVORITE: u32 = 2;
/// `ButtonRename`.
pub const PART_RENAME: u32 = 3;
/// `ButtonDelete`.
pub const PART_DELETE: u32 = 4;

/// The id of card `i`'s `part` — stable for as long as the list keeps its order
/// (the sort re-lists on a favourite toggle, and the ids follow the rows).
pub const fn card_part(card: usize, part: u32) -> u32 {
    CARD_BASE + card as u32 * PARTS + part
}
