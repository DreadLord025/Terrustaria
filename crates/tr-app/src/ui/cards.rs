//! The shared pieces of the two card-list screens (`ui/char_select.rs` ≅
//! `ui/world_select.rs` — the character and world cards mirror each other, the
//! screen ctor twins of `UIWorldSelect.cs:38-88`): the ctor-order-to-sheet
//! mapper, the 3-piece inner info panel, the card image button and the
//! `UITextPanel` styles. The per-screen halves (the `CardLayout` geometry, the
//! `CardAction` click tables, `draw`/`card_hit`) stay with their screens —
//! only the mirrored drawing primitives live here.

use tr_render::{SheetRect, SpriteRenderer};

use super::char_select::{BUTTON_FAVORITE, BUTTON_PLAY, BUTTON_RENAME};
use super::uistate;
use crate::assets;

/// The sheet index into `Lookups::card_buttons` (assets.rs:956-963 —
/// `[ButtonPlay, ButtonFavoriteActive, ButtonFavoriteInactive, ButtonRename,
/// ButtonDelete]`) for the ctor's append-order ordinals (`UICharacterListItem.cs:
/// 71-121`, `UIWorldListItem.cs:94-158`). The `BUTTON_*` ordinals key
/// `char_select::button_label`'s table, but the sheet array is a different
/// order: the favourite owns two sheets, so every button after it sits one
/// later (`BUTTON_RENAME` = 2, sheet 3; the delete ordinal lands on sheet 4
/// through the catch-all arm). Indexing the array with a raw ordinal drew the
/// wrong sprite (Task 3b review fix 1), so every card button draw resolves its
/// sheet through here.
pub(crate) fn button_sheet(button: usize, is_favorite: bool) -> usize {
    match button {
        BUTTON_PLAY => 0,                    // `ButtonPlay` (`UICharacterListItem.cs:71-79`, `UIWorldListItem.cs:95-103`)
        BUTTON_FAVORITE if is_favorite => 1, // `ButtonFavoriteActive` (`:80-87`, `:104-112`)
        BUTTON_FAVORITE => 2,                // `ButtonFavoriteInactive`
        BUTTON_RENAME => 3,                  // `ButtonRename` (`:101-107`, `:137-144`)
        _ => 4,                              // `ButtonDelete` (`:109-121`, `:146-158`)
    }
}

/// `DrawPanel` (`UICharacterListItem.cs:310-315`, `UIWorldListItem.cs:487-492`):
/// the card info panels are **not** `Utils.DrawSplicedPanel`'s 9 pieces — a
/// 3-piece horizontal slice of `InnerPanelBackground`: an 8 px left edge, the
/// 8 px middle stretched to `width - 16`, an 8 px right edge, every source rect
/// the sheet's **full height** (`this._innerPanelTexture.Height()` = 27). The
/// rect heights are placeholders filled with the sheet height at draw time,
/// like `panel::separator_rects`. (`char_select` re-exports this — it was that
/// module's pub surface before the dedup.)
pub fn inner_panel_rects(x: f32, y: f32, width: f32) -> [[f32; 4]; 3] {
    [
        [x, y, 8.0, 0.0],                // `:312`/`:489` — src `(0, 0, 8, H)`
        [x + 8.0, y, width - 16.0, 0.0], // `:313`/`:490` — src `(8, 0, 8, H)`, x-stretched
        [x + width - 8.0, y, 8.0, 0.0],  // `:314`/`:491` — src `(16, 0, 8, H)`
    ]
}

/// The [`inner_panel_rects`] draw. `None` sheet draws nothing (Review Focus 7).
pub(crate) fn draw_inner_panel(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    width: f32,
) {
    let Some(sheet) = lookups.ui_inner_panel else {
        return;
    };
    let h = sheet.height as f32; // `:312-314`/`:489-491`'s full-height sources
    let srcs: [[f32; 4]; 3] = [
        [0.0, 0.0, 8.0, h],
        [8.0, 0.0, 8.0, h],
        [16.0, 0.0, 8.0, h],
    ];
    for (dst, src) in inner_panel_rects(x, y, width).into_iter().zip(srcs) {
        if dst[2] <= 0.0 {
            continue;
        }
        let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
        let x0 = sheet.x as f32 + src[0];
        let y0 = sheet.y as f32 + src[1];
        let uv = [x0 / aw, y0 / ah, (x0 + src[2]) / aw, (y0 + src[3]) / ah];
        r.push_ui_sprite([dst[0], dst[1]], [dst[2], h], uv, [1.0; 4]);
    }
}

/// One card button: `UIImageButton.DrawSelf` (`UIImageButton.cs:48-56`) — the
/// sheet tinted `Color * (hover ? _visibilityActive : _visibilityInactive)`
/// with the visibility pair `SetVisibility` set (`UICharacterListItem.cs:86`
/// gives the favourite its 0.8 idle; `UIWorldListItem.cs:110` the world's); the
/// card buttons have no border texture, so a hover just draws the base at full
/// tint.
pub(crate) fn draw_card_button(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<SheetRect>,
    rect: [f32; 4],
    hover: bool,
    idle_alpha: f32,
) {
    let a = if hover { 1.0 } else { idle_alpha };
    if let Some(sheet) = sheet {
        super::push_whole(r, lookups, &sheet, rect, [a, a, a, a]);
    }
}

/// The `UITextPanel` header style (`UICharacterSelect.cs:59-64`,
/// `UIWorldSelect.cs:62-66`): the 0.8-large, 15-padded `(73, 94, 171)` panel.
/// Idle backgrounds are the `FadedMouseOut` restore (`UITextPanel.cs:129-133`);
/// the hover swap to `(73,94,171)`/`FancyUIFatButtonMouseOver` is
/// `uistate::draw_text_panel`'s.
pub(crate) fn header_style() -> uistate::TextPanelStyle {
    uistate::TextPanelStyle {
        text_scale: 0.8,
        large: true,
        padding: 15.0,
        bg: uistate::FADED_MOUSE_OVER_BG,
        border: uistate::TEXT_PANEL_BORDER,
    }
}

/// The 0.7-large, default-12-padded fat Back/New(/Create) panel style
/// (`UICharacterSelect.cs:65-84`, `UIWorldSelect.cs:68-86`,
/// `UIWorldCreation.cs:637-662`, `UICharacterCreation.cs:1177-1197`; a
/// `UITextPanel` without `SetPadding` keeps `UIPanel.cs:30-33`'s 12).
pub(crate) fn fat_button_style() -> uistate::TextPanelStyle {
    uistate::TextPanelStyle {
        text_scale: 0.7,
        large: true,
        padding: 12.0,
        bg: uistate::TEXT_PANEL_BG,
        border: uistate::TEXT_PANEL_BORDER,
    }
}
