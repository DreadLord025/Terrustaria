//! The Select Player screen (`Main.menuMode == 888`'s character half): a faithful
//! immediate port of `UICharacterSelect` + `UICharacterListItem` + the
//! `UICharacter` preview chrome.
//!
//! Same shape as `ui/crafting.rs`: a pure [`layout`] resolving the source's
//! `StyleDimension`s through the shared `uistate::resolve` pass, one
//! [`card_layout`] behind both the hit-tests and the draw (Review Focus 2), and
//! a [`draw`] that replays the source's child order. Nothing panics with
//! `Lookups::default()` — every sheet is an `Option` guard (Review Focus 7).
//!
//! The player sprite inside the 59x58 preview is Task 7's `draw_player_preview`;
//! until then the preview draws the `PlayerBackground` only and the sprite draw
//! site is a documented no-op (plan Task 3 step 3).
//!
//! Every pub item here is live since Task 5: the App-flow surface drives
//! `App.front = FrontScreen::CharSelect` (`main.rs` — `resolved_layout` +
//! `hit` for the clicks, `draw` for the frame, `sort_players` for the re-list).

use glam::Vec2;
use tr_render::SpriteRenderer;

use super::select_hit;
use super::color;
use super::hover_pair;
use super::in_rect;
use super::panel;
use super::push_whole;
use super::save_list;
use super::text::{border_string, hud_text_width, large_text_width};
use super::uistate::{self, Style};
use crate::assets;

// --- Source strings (Localization `UI.*` / `GameUI.*`, English) ---------------

/// `UI.SelectPlayer` — the header text (`UICharacterSelect.cs:59`).
pub fn select_player() -> String {
    crate::lang::text("UI.SelectPlayer")
}
/// `UI.Back` (`UICharacterSelect.cs:65`).
pub fn back() -> String {
    crate::lang::text("UI.Back")
}
/// `UI.New` (`UICharacterSelect.cs:76`).
pub fn new_button() -> String {
    crate::lang::text("UI.New")
}
/// `GameUI.PlayerLifeMax` — `" HP"` (`UICharacterListItem.cs:338`).
pub fn life_suffix() -> String {
    crate::lang::text("GameUI.PlayerLifeMax")
}
/// `GameUI.PlayerManaMax` — `" MP"` (`UICharacterListItem.cs:342`).
pub fn mana_suffix() -> String {
    crate::lang::text("GameUI.PlayerManaMax")
}
/// The fixed play-time string: the port tracks no play time, so `num5 = 0` and
/// `playTime` is zero — `"0" + 0 + ":00:00"` (`:376-379`; plan disposition 2).
pub const PLAYTIME_TEXT: &str = "00:00:00";

// --- Card metrics ------------------------------------------------------------

/// The card height, `base.Height.Set(96f, 0f)` (`UICharacterListItem.cs:61`).
pub const CARD_H: f32 = 96.0;
/// `this._playerList.ListPadding = 5f` (`UICharacterSelect.cs:52`) — the row
/// pitch is [`CARD_H`] + this (`UIList.cs:120-128`'s running offset).
pub const LIST_PADDING: f32 = 5.0;
/// One row's vertical step: 96 + 5 = 101.
pub const CARD_PITCH: f32 = CARD_H + LIST_PADDING;
/// `UIPanel.cs:32`'s default `SetPadding(12)` — the container panel's inner rect.
pub const PANEL_PADDING: [f32; 4] = [12.0; 4];
/// `base.SetPadding(6f)` (`UICharacterListItem.cs:63`) — the card's inner rect.
pub const CARD_PADDING: [f32; 4] = [6.0; 4];

/// The card buttons in ctor order (`UICharacterListItem.cs:71-121`); the cloud
/// button (:89-100) is skipped — `SocialAPI.Cloud == null` (plan §J.2). These
/// ordinals key only the [`button_label`] table — the `card_buttons` sheet
/// array is a different order, mapped by [`button_sheet`].
pub const BUTTON_PLAY: usize = 0;
/// The favourite toggle (`:80-87`).
pub const BUTTON_FAVORITE: usize = 1;
/// The rename button (`:101-107`).
pub const BUTTON_RENAME: usize = 2;
/// The delete button (`:109-121`) — right-aligned, label on the right.
pub const BUTTON_DELETE: usize = 3;

// --- Colours -------------------------------------------------------------------

/// `uIPanel.BackgroundColor = new Color(33, 43, 79) * 0.8f`
/// (`UICharacterSelect.cs:46`); XNA truncation `(26, 34, 63, 204)`.
pub const CONTAINER_PANEL_BG: [u8; 4] = [26, 34, 63, 204];
/// `base.BorderColor = new Color(89, 116, 213) * 0.7f`
/// (`UICharacterListItem.cs:51`, restored by `MouseOut` `:306`) — the rest card
/// border; XNA `(byte)(213*0.7) = 149`.
pub const CARD_BORDER_REST: [u8; 4] = [62, 81, 149, 178];
/// `new Color(89, 116, 213)` (`UICharacterListItem.cs:298`) — the hovered card border.
pub const CARD_BORDER_HOVER: [u8; 4] = [89, 116, 213, 255];
/// `new Color(63, 82, 151) * 0.7f` (`UICharacterListItem.cs:305`) — the rest card
/// background, the same truncation as `UIPanel`'s default (`uistate::TEXT_PANEL_BG`).
pub const CARD_BG_REST: [u8; 4] = [44, 57, 105, 178];
/// `Color.Gray` (`UICharacterListItem.cs:327`) — the un-loadable player's name.
pub const GRAY: [u8; 4] = [128, 128, 128, 255];

// --- Screen layout -------------------------------------------------------------

/// Every static rect of the screen, resolved through `uistate::resolve` exactly
/// as the source's construction order does (`UICharacterSelect.cs:35-86`).
/// `list`'s width carries the caller's `list_w` (626 detached / 601 attached);
/// `scrollbar` is the attach position — it only exists as a panel child while
/// [`scrollbar_attaches`] holds (`Recalculate`, `:88-106`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// The 80%-wide, 650-max root (`:37-42`).
    pub root: [f32; 4],
    /// The cyan container panel, `Height (-110, 1)` (`:43-48`).
    pub panel: [f32; 4],
    /// The `UIList`, `Width (0, 1)` / `(-25, 1)` (`:49-53`, `:96`, `:102`).
    pub list: [f32; 4],
    /// The `UIScrollbar` attach rect — `Width 20` (`UIScrollbar.cs:58-59`),
    /// `HAlign 1` of the panel inner, `Height (0, 1)` (`:54-58`).
    pub scrollbar: [f32; 4],
    /// The `Select Player` header — sized only by `SetText`'s minimums
    /// (`:59-64`, `UITextPanel.cs:132-134`).
    pub header: [f32; 4],
    /// The fat Back button (`:65-75`).
    pub back: [f32; 4],
    /// The fat New button — `CopyStyle(back)` + `HAlign 1` (`:76-84`).
    pub new: [f32; 4],
    /// `UIScrollbar.ViewPosition` / `UIList`'s `_innerList.Top` (`UIList.cs:181`):
    /// the list's scroll offset in px. `0` is the top; [`max_scroll`] is the
    /// bottom. Both the row rects (via [`Layout::inner_list`]) and the
    /// scrollbar handle ([`panel::scrollbar_handle_at`]) read it, so a scrolled
    /// row and its hit-test stay one rect (Review Focus 2).
    pub scroll: f32,
}

impl Layout {
    /// `UIInnerList`'s translated rect — `_innerList.Top.Set(0f - GetValue(), 0f)`
    /// (`UIList.cs:181`): the rows ride at `list.y - scroll` while the list
    /// itself stays the viewport the draw clips and the hit-test filters by
    /// (`UIList.cs:59`'s `OverflowHidden`, `framework.rs`'s `viewport`).
    pub fn inner_list(&self) -> [f32; 4] {
        [
            self.list[0],
            self.list[1] - self.scroll,
            self.list[2],
            self.list[3],
        ]
    }

    /// The screen with `scroll` applied and clamped by `UIList.UpdateScrollbar`'s
    /// `SetView` (`UIList.cs:130-137` -> `UIScrollbar.cs:72-78`): the view is the
    /// list's inner height, the max view the content height, and the position is
    /// pinned into `[0, maxViewSize - viewSize]` (`UIScrollbar.cs:75`).
    pub fn with_scroll(&self, scroll: f32, cards: usize) -> Layout {
        let mut l = *self;
        l.scroll = clamp_scroll(scroll, l.list[3], list_content_height(cards));
        l
    }
}

/// The root element's rect (`UICharacterSelect.cs:37-42`): `Width (0, 0.8)`,
/// `MaxWidth 650`, `Top 220`, `Height (-220, 1)`, `HAlign 0.5`.
fn root_rect(screen: Vec2) -> [f32; 4] {
    uistate::resolve(
        [0.0, 0.0, screen.x, screen.y],
        Style::px(0.0),          // Left unset (`StyleDimension.Empty`)
        Style::px(220.0),        // `uIElement.Top.Set(220f, 0f)` (`:40`)
        Style::new(0.0, 0.8),    // `Width.Set(0f, 0.8f)` (`:38`)
        Style::new(-220.0, 1.0), // `Height.Set(-220f, 1f)` (`:41`)
        0.5,                     // `HAlign = 0.5f` (`:42`)
        0.0,
        0.0,
        0.0,
        650.0, // `MaxWidth.Set(650f, 0f)` (`:39`)
        [0.0; 4],
    )
}

/// The container panel's rect (`UICharacterSelect.cs:43-48`): `Width (0, 1)`,
/// `Height (-110, 1)` of the root.
fn panel_rect(root: [f32; 4]) -> [f32; 4] {
    uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(0.0),
        Style::new(0.0, 1.0),   // `uIPanel.Width.Set(0f, 1f)` (`:44`)
        Style::new(-110.0, 1.0), // `uIPanel.Height.Set(-110f, 1f)` (`:45`)
        0.0,
        0.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    )
}

/// The whole screen: `header_w` is the measured `Select Player` width (Death
/// font, unscaled — the only font-dependent input, the P3 `title_width`
/// convention) and `list_w` the `UIList` width (626 detached at 1600x900, 601
/// when the scrollbar is attached — [`detached_list_width`] /
/// [`attach_scrollbar`]).
///
/// The header has no `Width`/`Height` — `SetText`'s minimums are its size
/// (`UITextPanel.cs:132-134`): `MinWidth = header_w*0.8 + PaddingLeft +
/// PaddingRight` (the `SetPadding(15)` of `:62`) and `MinHeight =
/// (32 if large)*0.8 + 30` = 55.6, so a 650-wide hint would be wrong — the
/// MinWidth-sized header is the lesson of the T1 review.
pub fn layout(screen: Vec2, header_w: f32, list_w: f32) -> Layout {
    let root = root_rect(screen);
    let panel = panel_rect(root);
    // The list and the scrollbar resolve against the panel's INNER dimensions
    // (`UIPanel.cs:32`'s padding 12; `UIElement.cs:287-288`).
    let panel_inner = uistate::inner(panel, PANEL_PADDING);
    // `_playerList`: `Width (0, 1)` / `(-25, 1)` (`:50`, `:96`, `:102`),
    // `Height (0, 1)` (`:51`) — the caller's `list_w` carries the branch.
    let list = [panel_inner[0], panel_inner[1], list_w, panel_inner[3]];
    // `_scrollbar`: `Width.Set(20f)` + `MaxWidth 20` (`UIScrollbar.cs:58-59`),
    // `HAlign = 1f` (`:57`), `Height (0, 1)` (`:56`).
    let scrollbar = uistate::resolve(
        panel_inner,
        Style::px(0.0),
        Style::px(0.0),
        Style::px(20.0),
        Style::new(0.0, 1.0),
        1.0,
        0.0,
        0.0,
        0.0,
        20.0,
        [0.0; 4],
    );
    // The header: `HAlign 0.5`, `Top -40`, `SetPadding(15)`, no Width/Height —
    // `SetText`'s minimums size it (`:59-64`, `UITextPanel.cs:132-134`).
    let header = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-40.0), // `uITextPanel.Top.Set(-40f, 0f)` (`:61`)
        Style::px(0.0),   // no Width set — the hint is 0 wide
        Style::px(0.0),   // no Height set
        0.5,              // `uITextPanel.HAlign = 0.5f` (`:60`)
        0.0,
        // `MinWidth = _textSize.X + PaddingLeft + PaddingRight`
        // (`UITextPanel.cs:133`): `header_w * 0.8 + 15 + 15`.
        header_w * 0.8 + 30.0,
        // `MinHeight = (32 if large)*scale + PaddingTop + PaddingBottom`
        // (`UITextPanel.cs:134`): `32*0.8 + 30`.
        32.0 * 0.8 + 30.0,
        root[2],
        [0.0; 4],
    );
    // `_backPanel`: `Width (-10, 0.5)`, `Height 50`, `VAlign 1`, `Top -45`
    // (`:66-69`) — bottom of the root, 45 px up.
    let back = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-45.0),
        Style::new(-10.0, 0.5),
        Style::px(50.0),
        0.0,
        1.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    );
    // `_newPanel`: `CopyStyle(back)` + `HAlign 1` (`:77-78`).
    let new = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(-45.0),
        Style::new(-10.0, 0.5),
        Style::px(50.0),
        1.0,
        1.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],
    );
    Layout {
        root,
        panel,
        list,
        scrollbar,
        header,
        back,
        new,
        // A freshly built screen is unscrolled; the wheel and the scrollbar
        // write the offset through [`Layout::with_scroll`].
        scroll: 0.0,
    }
}

/// The detached `UIList` width: `Width.Set(0f, 1f)` (`:50`, `:96`) over the
/// panel inner — 626 at 1600x900.
pub fn detached_list_width(screen: Vec2) -> f32 {
    uistate::inner(panel_rect(root_rect(screen)), PANEL_PADDING)[2]
}

/// The `UIList` content height (`UIList.cs:120-128`): every row's outer height
/// plus one `ListPadding` per row (`:122`'s per-item `num2`; the single-row list
/// drops it). N cards = N*(96+5); one card = 96; none = 0.
pub fn list_content_height(cards: usize) -> f32 {
    match cards {
        0 => 0.0,
        1 => CARD_H,
        n => n as f32 * CARD_PITCH,
    }
}

/// `UserInterface`'s wheel accumulator is XNA's raw counter, so one detent is
/// 120 units (`PlayerInput.cs:898-899` copies `ScrollWheelValue -
/// ScrollWheelValueOld` into `ScrollWheelDeltaForUI`) — and `UIList.ScrollWheel`
/// spends it 1:1 on `ViewPosition` (`UIList.cs:112`). The port's pointer machine
/// carries notches (winit `LineDelta`), so the px travel is `notches * 120`.
pub const WHEEL_STEP: f32 = 120.0;

/// The scroll range — `UIScrollbar.SetView`'s clamp upper bound
/// `maxViewSize - viewSize` (`UIScrollbar.cs:75`): the content height the
/// `UIList` reports (`UIList.cs:130-137`) minus the list's own inner height.
pub fn max_scroll(view_h: f32, content_h: f32) -> f32 {
    (content_h - view_h).max(0.0)
}

/// [`max_scroll`] as a clamp — `ViewPosition`'s setter (`UIScrollbar.cs:43-46`)
/// and `SetView` (`:75`) pin the offset into the same range.
pub fn clamp_scroll(scroll: f32, view_h: f32, content_h: f32) -> f32 {
    scroll.clamp(0.0, max_scroll(view_h, content_h))
}

/// `UIList.ScrollWheel` (`UIList.cs:107-114`): `_scrollbar.ViewPosition -=
/// evt.ScrollWheelValue`. The port's `delta` is in notches (positive = wheel
/// up), so one notch up walks the view back [`WHEEL_STEP`] px (`:112`'s `-=`),
/// clamped by `SetView` (`UIScrollbar.cs:75`).
pub fn scroll_wheel(scroll: f32, delta: i32, view_h: f32, content_h: f32) -> f32 {
    clamp_scroll(scroll - delta as f32 * WHEEL_STEP, view_h, content_h)
}

/// Whether the wheel reaches the list at `mouse` — `UserInterface.Update`
/// dispatches the wheel to `GetElementAt` (`:219-221`), and the `UIList`'s inner
/// list answers `ContainsPoint` with `true` (`UIList.cs:16-19`), so the whole
/// list rect (gaps included) scrolls, and the attached `UIScrollbar`
/// (`:42`) is its own target.
pub fn wheel_target(l: &Layout, cards: usize, mouse: Vec2) -> bool {
    let content_h = list_content_height(cards);
    in_rect(mouse, l.list)
        || (scrollbar_attaches(l.list[3], content_h) && in_rect(mouse, l.scrollbar))
}

/// What a press on the attached scrollbar does (`UIScrollbar.LeftMouseDown`,
/// `UIScrollbar.cs:126-143`): `(new view position, drag latch)`. `Some(offset)`
/// is the `_isDragging` half — `_dragYOffset = mouse.Y - handle.Y` (`:134-135`);
/// `None` is the track jump, which needs no latch (`:137-141`). `None` as a whole
/// means the press was not the scrollbar's (detached bar, or outside it).
pub fn scrollbar_press(l: &Layout, cards: usize, mouse: Vec2) -> Option<(f32, Option<f32>)> {
    let content_h = list_content_height(cards);
    if !scrollbar_attaches(l.list[3], content_h) || !in_rect(mouse, l.scrollbar) {
        return None;
    }
    let handle = panel::scrollbar_handle_at(l.scrollbar, l.list[3], content_h, l.scroll);
    if in_rect(mouse, handle) {
        // `:134-135` — the grab keeps the pointer's offset inside the handle.
        return Some((l.scroll, Some(mouse.y - handle[1])));
    }
    // `:140` — a track press centres the handle under the pointer.
    Some((
        panel::scrollbar_jump_value(l.scrollbar, l.list[3], content_h, handle[3], mouse.y),
        None,
    ))
}

/// The drag frame (`UIScrollbar.DrawSelf`, `UIScrollbar.cs:109-112`): while
/// `_isDragging` the position follows the pointer with the latched offset.
pub fn scrollbar_drag(l: &Layout, cards: usize, drag_offset: f32, mouse_y: f32) -> f32 {
    let content_h = list_content_height(cards);
    panel::scrollbar_drag_value(l.scrollbar, l.list[3], content_h, drag_offset, mouse_y)
}

/// The `Recalculate` scrollbar branch's `CanScroll` half (`UICharacterSelect.cs:
/// 92/98`): `UIList.UpdateScrollbar` (`UIList.cs:130-136`) calls
/// `SetView(viewHeight, innerListHeight)`, whose clamp pins the view into
/// `[0, maxViewSize]` (`UIScrollbar.cs:78`) before `CanScroll` compares
/// (`UIScrollbar.cs:49`) — so the clamped-equal cases (empty list, one card, a
/// short list) never scroll and the 546 px view scrolls from 6 cards up
/// (Review Focus 3).
pub fn scrollbar_attaches(view_h: f32, content_h: f32) -> bool {
    panel::scrollbar_can_scroll(view_h.min(content_h), content_h)
}

/// The `Recalculate` branch itself (`UICharacterSelect.cs:88-106`): attaching
/// flips the list width to `Width.Set(-25f, 1f)` (`:102`) — the detached width
/// this `Layout` carries shrinks by 25 — and the scrollbar becomes a panel
/// child at `l.scrollbar`; detaching restores `Width.Set(0f, 1f)` (`:96`).
/// The branch is ported and unit-tested on a synthetic tall list, never
/// deleted (Review Focus 3).
pub fn attach_scrollbar(mut l: Layout, attached: bool) -> Layout {
    if attached {
        l.list[2] -= 25.0; // `:102`
    }
    l
}

/// The screen as `Recalculate` leaves it for `cards` cards: [`layout`] at the
/// detached width plus the [`scrollbar_attaches`] branch. `draw` and `card_hit`
/// both build the screen through this so their geometry agrees by construction
/// (Review Focus 2).
pub fn resolved_layout(screen: Vec2, header_w: f32, cards: usize) -> Layout {
    let l = layout(screen, header_w, detached_list_width(screen));
    attach_scrollbar(l, scrollbar_attaches(l.list[3], list_content_height(cards)))
}

// --- Card geometry -------------------------------------------------------------

/// One card's geometry (Review Focus 2's single layout function). `row_rect` is
/// the card list's rect — the `UIList` the 96 px rows stack in — and `index`
/// picks the row: card i sits at `(list.x, list.y + i*101, list.w, 96)` (the
/// 96 px card, `UICharacterListItem.cs:61`, plus one 5 px `ListPadding` per row,
/// `UICharacterSelect.cs:52`). Every button/label/panel below derives from that
/// one rect, so the hit-tests and the draw cannot drift apart.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CardLayout {
    /// The card `UIPanel` itself — `Width (0, 1)`, `Height 96`, `SetPadding(6)`
    /// (`:61-63`).
    pub card: [f32; 4],
    /// `UICharacter` 59x58, `Left 4` (`:66-67`, `UICharacter.cs:43-44`).
    pub preview: [f32; 4],
    /// `ButtonPlay` 22x22, `Left 4`, `VAlign 1` (`:71-79`).
    pub play: [f32; 4],
    /// `ButtonFavorite*` 22x22, `Left 28` (`:80-88`).
    pub favorite: [f32; 4],
    /// `ButtonRename` 22x22, `Left 52` — the cloud button is skipped
    /// (`SocialAPI.Cloud == null`, `:89-100`), so rename takes its 52.
    pub rename: [f32; 4],
    /// `ButtonDelete` 22x22, `HAlign 1` + `VAlign 1` (`:109-113`).
    pub delete: [f32; 4],
    /// The hover label, `Left num` where the ctor's `num` is
    /// 4 + 24 + 24 + 24 + 4 = 80 (`:71-122`), `Top -3`, `VAlign 1` (`:122-127`);
    /// the empty `UIText` is 0 wide and 16 tall (`UIText.cs:186-189`).
    pub label: [f32; 4],
    /// The delete hover label, `Left -30`, `HAlign 1`, `Top -3`, `VAlign 1`
    /// (`:128-133`) — its right edge is the anchor; the text grows leftward.
    pub delete_label: [f32; 4],
    /// `DrawBorderString` name position: `num + 6, dimensions.Y - 2` where `num`
    /// is the preview's right edge (`:322`, `:330`).
    pub name_pos: [f32; 2],
    /// The `Divider` strip at `(num, inner.Y + 21)`, stretched to the card's
    /// **outer** right edge (`:331` and the plan's correction — the width source
    /// is `GetDimensions()`, not the inner one), 8x4 texture.
    pub divider: [f32; 4],
    /// The life/mana panel: `num + 6, inner.Y + 29`, 200 px, the 27 px
    /// `InnerPanelBackground` height (`:332-335`).
    pub life_panel: [f32; 4],
    /// `TextureAssets.Heart` 22x22 at `+ (5, 2)` (`:336`).
    pub heart: [f32; 4],
    /// The life text: `+ 10 + 22` on x, `+3` on y (`:337-338`).
    pub life_text: [f32; 2],
    /// `vector2.X += 65`, then `Mana` 22x24 at `+ (5, 2)` (`:339-340`).
    pub mana_icon: [f32; 4],
    /// The mana text: `+ 10 + 24` on x, `+3` on y (`:341-342`).
    pub mana_text: [f32; 2],
    /// The difficulty panel: `+ (200 + 5)`, 140 px (English; the 180 px arm is
    /// the Russian culture, `:346-349`) (`:343-350`).
    pub difficulty_panel: [f32; 4],
    /// The play-time panel: the rest of the inner width (`:372-375`).
    pub playtime_panel: [f32; 4],
}

/// Row `index`'s card geometry. See [`CardLayout`] for the field-by-field cites.
pub fn card_layout(row_rect: [f32; 4], index: usize) -> CardLayout {
    let shift = index as f32 * CARD_PITCH; // 96 px card + 5 px ListPadding
    let card = [row_rect[0], row_rect[1] + shift, row_rect[2], CARD_H]; // `:61-62`
    let inner = uistate::inner(card, CARD_PADDING); // `SetPadding(6)` (`:63`)
    let (ix, iy, iw) = (inner[0], inner[1], inner[2]);
    let bottom = iy + inner[3]; // the `VAlign 1` anchor

    // `this._playerPanel.Left.Set(4f, 0f)` (`:67`); the 59x58 `UICharacter`
    // (`UICharacter.cs:43-44`), Top unset (0).
    let preview = [ix + 4.0, iy, 59.0, 58.0];
    let preview_right = preview[0] + preview[2]; // DrawSelf's `num` (`:322`)

    // The ctor's `num` walk (`:71-122`): 4 (play), +24 (favorite at 28), +24
    // (rename at 52; the cloud slot's +24 is skipped with `SocialAPI.Cloud`),
    // +24 (the delete slot), +4 -> the label at 80.
    let num = 4.0 + 24.0 + 24.0 + 24.0 + 4.0;
    // `UIText` is 16 tall at scale 1 and 0 wide while empty (`UIText.cs:186-189`).
    let label_h = 16.0;
    let label_y = bottom - label_h - 3.0; // `Top.Set(-3f)` + `VAlign 1`

    CardLayout {
        card,
        preview,
        // The 22x22 image buttons, `VAlign 1` (`:73/:81/:102/:111`).
        play: [ix + 4.0, bottom - 22.0, 22.0, 22.0],
        favorite: [ix + 28.0, bottom - 22.0, 22.0, 22.0],
        rename: [ix + 52.0, bottom - 22.0, 22.0, 22.0],
        // `HAlign = 1f` + `VAlign = 1f` (`:110-112`).
        delete: [ix + iw - 22.0, bottom - 22.0, 22.0, 22.0],
        label: [ix + num, label_y, 0.0, label_h],
        // `Left.Set(-30f)` + `HAlign 1` — the right edge anchors 30 px in.
        delete_label: [ix + iw - 30.0, label_y, 0.0, label_h],
        // `new Vector2(num + 6f, dimensions.Y - 2f)` (`:330`).
        name_pos: [preview_right + 6.0, preview[1] - 2.0],
        // `:331` — y = `innerDimensions.Y + 21f`, width to the card's OUTER
        // right (`GetDimensions()`), the 8x4 `Divider` scaled by `dx / 8f`.
        divider: [
            preview_right,
            iy + 21.0,
            card[0] + card[2] - preview_right,
            4.0,
        ],
        // `new Vector2(num + 6f, innerDimensions.Y + 29f)`, `num2 = 200f` (`:332-334`).
        life_panel: [preview_right + 6.0, iy + 29.0, 200.0, 27.0],
        // `vector2 + new Vector2(5f, 2f)`, the 22x22 Heart (`:336`).
        heart: [preview_right + 11.0, iy + 31.0, 22.0, 22.0],
        // `vector2.X += 10f + Heart.Width()` from the panel x, text `+3` (`:337-338`).
        life_text: [preview_right + 6.0 + 10.0 + 22.0, iy + 32.0],
        // `vector2.X += 65f`, `Mana` 22x24 at `+ (5, 2)` (`:339-340`).
        mana_icon: [preview_right + 6.0 + 32.0 + 65.0 + 5.0, iy + 31.0, 22.0, 24.0],
        // `vector2.X += 10f + Mana.Width()` (24), text `+3` (`:341-342`).
        mana_text: [preview_right + 6.0 + 32.0 + 65.0 + 10.0 + 24.0, iy + 32.0],
        // `vector.X += num2 + 5f`, `num3 = 140f` (English; RU would be 180,
        // `:345-349`).
        difficulty_panel: [preview_right + 6.0 + 205.0, iy + 29.0, 140.0, 27.0],
        // `vector.X += num3 + 5f`, `num4 = inner right - vector4.X` (`:372-374`).
        playtime_panel: [
            preview_right + 6.0 + 205.0 + 145.0,
            iy + 29.0,
            ix + iw - (preview_right + 6.0 + 205.0 + 145.0),
            27.0,
        ],
    }
}

// --- Sort / colours / labels -----------------------------------------------------

/// `UICharacterSelect.UpdatePlayersList` (`UICharacterSelect.cs:146-157`):
/// favourite desc, then `LastPlayed` (the port has no play times — the list's
/// insertion order stands in), then Name, then filename. T2 owns the
/// implementation; the re-export keeps one copy of the chain — the App's
/// re-list and the favourite toggle call it through this path (Task 5).
pub use super::save_list::sort_players;

/// Hover colours (`MouseOver` `:294-300` / `MouseOut` `:302-308`): the card
/// background/border pair as pre-tint bytes — hover `(73,94,171)` /
/// `(89,116,213)`, rest `(63,82,151)*0.7` / `(89,116,213)*0.7` (XNA byte
/// truncation). Both colours pick through `ui::hover_pair`.
pub fn card_colors(hovered: bool) -> ([u8; 4], [u8; 4]) {
    (
        hover_pair(CARD_BG_REST, uistate::FADED_MOUSE_OVER_BG, hovered),
        hover_pair(CARD_BORDER_REST, CARD_BORDER_HOVER, hovered),
    )
}

/// `DrawSelf`'s difficulty switch (`UICharacterListItem.cs:353-370`): the
/// `UI.Softcore/Mediumcore/Hardcore/Creative` label and its colour
/// (`White` / `Main.mcColor` / `Main.hcColor` / `Main.creativeModeColor`).
pub fn difficulty_text(difficulty: u8) -> (String, [u8; 4]) {
    match difficulty {
        1 => (crate::lang::text("UI.Mediumcore"), uistate::MC_COLOR),
        2 => (crate::lang::text("UI.Hardcore"), uistate::HC_COLOR),
        3 => (crate::lang::text("UI.Creative"), uistate::CREATIVE_MODE_COLOR),
        // `case 0` and the unlabelled fallthrough both land on Classic/White.
        _ => (crate::lang::text("UI.Softcore"), [255, 255, 255, 255]),
    }
}

/// `DrawSelf`'s play-time string (`UICharacterListItem.cs:376-378`):
/// `num5 = Days*24 + Hours`, zero-padded to two digits when under 10, then
/// `playTime.ToString("\:mm\:ss")` — `"HH:MM:SS"` with the hours riding past
/// 24 into the day count. The port pins `00:00:00` (no play-time tracking,
/// the documented disposition); this is the source's formatter for the
/// tracking's landing.
#[allow(dead_code)] // the draw pins PLAYTIME_TEXT until play-time tracking lands
pub fn playtime_text(total_seconds: u64) -> String {
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;
    let num5 = days * 24 + hours; // `:377`
    let pad = if num5 < 10 { "0" } else { "" }; // `:378`
    format!("{pad}{num5}:{minutes:02}:{seconds:02}")
}

// --- UICharacterListItem's missing halves (W4-d81 census row 17) ----------------
//
// The card geometry/colours/labels/hit table/draw are landed above; these port
// CompareTo, the rename-finish trim and the unplayable-name prefix.

/// `CompareTo` (`UICharacterListItem.cs:285-292`): another card compares by
/// `_orderInList` — the ctor's insertion index (`:49`, `:65`), the list's
/// build order, NOT the favourite/LastPlayed chain (that is
/// `UICharacterSelect.UpdatePlayersList`, [`sort_players`]).
pub fn compare_card_order(a: usize, b: usize) -> std::cmp::Ordering {
    a.cmp(&b) // `:289`
}

/// `OnFinishedSettingName` (`UICharacterListItem.cs:208-214`): the keyboard's
/// result is `name.Trim()` (`:210`) before `PlayerFileData.Rename` — C#
/// `string.Trim` strips Unicode whitespace both ends, like Rust's `trim`.
/// The rename itself + `menuMode = 10` + re-open are App flow (`:211-213`).
pub fn renamed_player_name(keyboard_result: &str) -> String {
    keyboard_result.trim().to_string() // `:210`
}

/// `StatusID.Search.GetName` (`ID/StatusID.cs:25-32` over
/// `IdDictionary.Create<StatusID, int>`, `ReLogic.Reflection/IdDictionary.cs:
/// 84-94` — the public static int fields' names keyed by value). `GetName`
/// throws on an unknown id (`IdDictionary.cs:45`); the port maps only the
/// seven declared ids and returns `None` otherwise.
pub fn load_status_name(status: i32) -> Option<&'static str> {
    Some(match status {
        0 => "Ok",                // `StatusID.cs:25`
        1 => "LaterVersion",      // `:26`
        2 => "UnknownError",      // `:27`
        3 => "EmptyFile",         // `:28`
        4 => "DecryptionError",   // `:29`
        5 => "BadSectionPointer", // `:30`
        6 => "BadFooter",         // `:31`
        _ => return None,
    })
}

/// `DrawSelf`'s name line (`UICharacterListItem.cs:324-330`): an Ok
/// `loadStatus` draws the plain name in White; anything else prefixes
/// `"(" + StatusID.Search.GetName(loadStatus) + ") "` (`:328`) and draws Gray.
/// Returns `(text, is_gray)`; the draw already picks [`GRAY`] on `!playable`
/// but carries no prefix yet (every entry is Ok today, save_list).
#[allow(dead_code)] // no .plr loadStatus reader yet (every entry is Ok)
pub fn card_name_text(name: &str, load_status: i32) -> (String, bool) {
    if load_status == 0 {
        (name.to_string(), false) // `:323-324`, Color.White
    } else {
        let status = load_status_name(load_status).unwrap_or("");
        (format!("({status}) {name}"), true) // `:327-328`, Color.Gray
    }
}

/// The hover-label table (`:140-194`): the `UI.*` string a hovered button
/// shows. The favourite arm flips (`:147-154`) and the delete arm is the
/// favourited gate's label (`:174-184`) — drawn on the right-hand
/// [`CardLayout::delete_label`], not the left one.
pub fn button_label(button: usize, is_favorite: bool) -> String {
    match button {
        BUTTON_PLAY => crate::lang::text("UI.Play"), // `UI.Play` (`:171`)
        BUTTON_FAVORITE => {
            if is_favorite {
                crate::lang::text("UI.Unfavorite") // `UI.Unfavorite` (`:149`)
            } else {
                crate::lang::text("UI.Favorite") // `UI.Favorite` (`:153`)
            }
        }
        BUTTON_RENAME => crate::lang::text("UI.Rename"), // `UI.Rename` (`:142`)
        _ => {
            if is_favorite {
                crate::lang::text("UI.CannotDeleteFavorited") // `UI.CannotDeleteFavorited` (`:178`)
            } else {
                crate::lang::text("UI.Delete") // `UI.Delete` (`:182`)
            }
        }
    }
}

// The sheet index into `Lookups::card_buttons` (assets.rs:956-963 —
// `[ButtonPlay, ButtonFavoriteActive, ButtonFavoriteInactive, ButtonRename,
// ButtonDelete]`) for the `BUTTON_*` ctor ordinals above — the key of
// [`button_label`]'s table — is `ui::cards::button_sheet`: the favourite owns
// two sheets, so every button after it sits one later (`BUTTON_RENAME` = 2,
// sheet 3). Indexing the array with a raw ordinal drew the wrong sprite
// (Task 3b review fix 1), so every card button draw resolves its sheet there.

// --- Hit-testing -----------------------------------------------------------------

/// What a click does (`UICharacterSelect.cs:108-121`, `UICharacterListItem.cs:
/// 196-254` + the card handlers). `Play` reaches the App only for a playable
/// entry (`loadStatus == Ok`, `:258`); `Rename` opens the virtual keyboard with
/// `Lang.menu[45]` "Enter Character Name:" (`:200`); `Delete` routes to the
/// Task 5 confirm (`menuMode = 5`, `:250`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CardAction {
    Play(usize),
    ToggleFavorite(usize),
    Rename(usize),
    Delete(usize),
    /// The favourited card's delete click: `:114-117` never *subscribes* the
    /// handler, so the click is dead — the hit-test surfaces that as
    /// `DeleteBlocked` instead of skipping the button (Review Focus 4), and the
    /// `Cannot Delete (Favorited)` label already explains why.
    DeleteBlocked,
    New,
    Back,
    /// `OnLeftDoubleClick` → `PlayGame` (`UICharacterListItem.cs:68-69`): the
    /// second press of the same card inside 500 ms.
    DoubleClickPlay(usize),
}

/// The screen's click table. `favorites` carries one flag per card — the
/// delete gate needs it (Review Focus 4), so it replaces the plan's bare
/// `cards: usize` (the count is `favorites.len()`). Returns `(hovered card,
/// action)`: a card body without a button is `(Some(i), None)`; `Back`/`New`
/// come back without a card. The caller passes a [`resolved_layout`] so the
/// attached-scrollbar card widths agree with the draw. A `double_click` press
/// resolves to the card's `DoubleClickPlay` — `UICharacterListItem.cs:68-69`'s
/// `OnLeftDoubleClick += PlayGame`, which fires on the second press of the same
/// element inside 500 ms and is the only double-click user in 1.4.5.8.
///
/// The order is `ui::framework::get_element_at` (`UIElement.cs:324-352`): the
/// elements are listed in the source's **append** order — the card bodies and
/// their buttons (`UICharacterListItem.cs:71-121`), then Back (`:75`), then New
/// (`:84`) — and the reverse walk makes the last-appended element take the
/// click, so wherever the rects overlap (the last card's rows run under the fat
/// buttons once the list scrolls, 6+ players) it is theirs, not the card's
/// (Task 3b review fix 2). New and Back are disjoint, so their relative order is
/// cosmetic; inside a card the buttons are disjoint too, and they precede the
/// body they sit on.
pub fn card_hit(
    l: &Layout,
    favorites: &[bool],
    mouse: Vec2,
    double_click: bool,
) -> (Option<usize>, Option<CardAction>) {
    let table = elements(l, favorites);
    let Some(id) = super::framework::get_element_at(&table, mouse) else {
        return (None, None);
    };
    if id == select_hit::NEW {
        return (None, Some(CardAction::New)); // `NewCharacterClick` (`:108-114`)
    }
    if id == select_hit::BACK {
        return (None, Some(CardAction::Back)); // `GoBackClick` (`:116-120`)
    }
    // The child→parent bubble (`UIElement.cs:444-447`): a click on one of the
    // card's buttons is raised on the button and then on its `UICharacterListItem`
    // panel, so the chain's last id is the card that owns the part.
    let chain = super::framework::bubble_chain(&table, id);
    let panel = *chain.last().expect("the chain starts at the hit id");
    let card = ((panel - select_hit::CARD_BASE) / select_hit::PARTS) as usize;
    let part = (id - select_hit::CARD_BASE) % select_hit::PARTS;
    if part == select_hit::PART_BODY {
        if double_click {
            return (Some(card), Some(CardAction::DoubleClickPlay(card)));
        }
        return (Some(card), None);
    }
    let action = match part {
        select_hit::PART_PLAY => Some(CardAction::Play(card)),
        select_hit::PART_FAVORITE => Some(CardAction::ToggleFavorite(card)),
        select_hit::PART_RENAME => Some(CardAction::Rename(card)),
        // The favourite-gated delete returns `DeleteBlocked` rather than
        // nothing (Review Focus 4).
        _ if favorites.get(card).copied().unwrap_or(false) => Some(CardAction::DeleteBlocked),
        _ => Some(CardAction::Delete(card)),
    };
    (Some(card), action)
}

/// The screen's flat element list in the C#'s append order. The rows carry the
/// `UIList`'s viewport: `GetElementAt` cannot descend into a row whose ancestors
/// do not contain the point (`UIElement.cs:327-341`) and the draw clips at the
/// same rect (`OverflowHidden`, `:222-237`), so a row scrolled fully past the
/// list's bottom edge is neither painted nor clickable, while a row straddling
/// the edge is half-painted and still hit above it.
fn elements(l: &Layout, favorites: &[bool]) -> Vec<super::framework::Element> {
    let mut out = Vec::with_capacity(favorites.len() * select_hit::PARTS as usize + 2);
    // The rows ride on the translated inner list (`UIList.cs:181`); the
    // viewport stays the `UIList`'s own rect (`UIList.cs:59`).
    let rows = l.inner_list();
    for i in 0..favorites.len() {
        let c = card_layout(rows, i);
        // The body first: the buttons are the card's appended children
        // (`UICharacterListItem.cs:71-121`), so the reverse walk tests them
        // before the panel they sit on. Each button carries the panel as its
        // `Parent` so a click on one bubbles to the card (`UIElement.cs:444-447`).
        let body = select_hit::card_part(i, select_hit::PART_BODY);
        out.push(super::framework::Element::row(c.card, body, l.list));
        out.push(super::framework::Element::row_child(
            c.play,
            select_hit::card_part(i, select_hit::PART_PLAY),
            body,
            l.list,
        ));
        out.push(super::framework::Element::row_child(
            c.favorite,
            select_hit::card_part(i, select_hit::PART_FAVORITE),
            body,
            l.list,
        ));
        out.push(super::framework::Element::row_child(
            c.rename,
            select_hit::card_part(i, select_hit::PART_RENAME),
            body,
            l.list,
        ));
        out.push(super::framework::Element::row_child(
            c.delete,
            select_hit::card_part(i, select_hit::PART_DELETE),
            body,
            l.list,
        ));
    }
    out.push(super::framework::Element::hit(l.back, select_hit::BACK));
    out.push(super::framework::Element::hit(l.new, select_hit::NEW));
    out
}

/// The element under the pointer — `UIElement.GetElementAt` (`:324-352`).
pub fn hit_element(l: &Layout, favorites: &[bool], mouse: Vec2) -> Option<u32> {
    super::framework::get_element_at(&elements(l, favorites), mouse)
}

/// The flat form of [`card_hit`]: just the action, if any.
pub fn hit(l: &Layout, favorites: &[bool], mouse: Vec2) -> Option<CardAction> {
    card_hit(l, favorites, mouse, false).1
}

// --- Draw ------------------------------------------------------------------------

/// The card pieces shared with `ui/world_select.rs` (the mirrored
/// `UIWorldListItem` card): the `UITextPanel` styles, the 3-piece
/// `DrawPanel` info panel (`UICharacterListItem.cs:310-315`), the image
/// button and the ctor-order-to-sheet mapper. `inner_panel_rects` stays
/// re-exported — it was this module's pub surface before the dedup.
pub use super::cards::inner_panel_rects;

use super::cards::{button_sheet, draw_card_button, draw_inner_panel, fat_button_style, header_style};

/// One card, in the source's z-order (`UIElement.cs:199-230`): the card's own
/// `DrawSelf` first (the `UIPanel` slice, then the name/divider/info-panel
/// extras of `:317-380`), then the appended children — the preview and the
/// buttons/labels.
fn draw_card(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    player_assets: &crate::render::entities::PlayerAssets,
    l: &Layout,
    index: usize,
    entry: &save_list::PlayerEntry,
    mouse: Vec2,
) {
    let c = card_layout(l.inner_list(), index);
    let hovered = in_rect(mouse, c.card); // `MouseOver`/`MouseOut` (`:294-308`)
    let (bg, border) = card_colors(hovered);
    // `UIPanel.DrawSelf` (`UIPanel.cs:68-76`): background, then border.
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, c.card, color::tint(bg));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, c.card, color::tint(border));

    // DrawSelf extras (`:317-380`).
    // The name: `DrawBorderString` at `(num + 6, dimensions.Y - 2)`; a non-Ok
    // `loadStatus` draws Gray (`:325-329`) — the port's `playable` is that
    // flag, and every entry today is Ok (no `.plr` reader yet, save_list), so
    // the Gray arm carries no status prefix.
    let (name_tint, name) = if entry.playable {
        ([1.0; 4], entry.name.as_str())
    } else {
        (color::tint(GRAY), entry.name.as_str())
    };
    border_string(r, lookups, c.name_pos[0], c.name_pos[1], name_tint, name);
    // The `Divider` 8x4 stretched to the card's outer right (`:331`).
    if let Some(sheet) = lookups.ui_divider {
        push_whole(r, lookups, &sheet, c.divider, [1.0; 4]);
    }
    // Life/mana panel + icons + texts (`:332-342`).
    draw_inner_panel(r, lookups, c.life_panel[0], c.life_panel[1], c.life_panel[2]);
    if let Some(sheet) = lookups.ui_heart {
        push_whole(r, lookups, &sheet, c.heart, [1.0; 4]);
    }
    border_string(
        r,
        lookups,
        c.life_text[0],
        c.life_text[1],
        [1.0; 4],
        &format!("{}{}", entry.life_max, life_suffix()),
    );
    if let Some(sheet) = lookups.ui_mana {
        push_whole(r, lookups, &sheet, c.mana_icon, [1.0; 4]);
    }
    border_string(
        r,
        lookups,
        c.mana_text[0],
        c.mana_text[1],
        [1.0; 4],
        &format!("{}{}", entry.mana_max, mana_suffix()),
    );
    // Difficulty panel (`:343-371`), text centred at `+ (num3/2 - w/2, 3)`.
    draw_inner_panel(
        r,
        lookups,
        c.difficulty_panel[0],
        c.difficulty_panel[1],
        c.difficulty_panel[2],
    );
    let (diff_text, diff_color) = difficulty_text(entry.difficulty);
    let diff_w = hud_text_width(r, lookups, &diff_text, 1.0);
    border_string(
        r,
        lookups,
        c.difficulty_panel[0] + c.difficulty_panel[2] * 0.5 - diff_w * 0.5,
        c.difficulty_panel[1] + 3.0,
        color::tint(diff_color),
        &diff_text,
    );
    // Play-time panel (`:372-379`) — the fixed `00:00:00` (no play-time
    // tracking, plan disposition 2), centred the same way.
    draw_inner_panel(
        r,
        lookups,
        c.playtime_panel[0],
        c.playtime_panel[1],
        c.playtime_panel[2],
    );
    let time_w = hud_text_width(r, lookups, PLAYTIME_TEXT, 1.0);
    border_string(
        r,
        lookups,
        c.playtime_panel[0] + c.playtime_panel[2] * 0.5 - time_w * 0.5,
        c.playtime_panel[1] + 3.0,
        [1.0; 4],
        PLAYTIME_TEXT,
    );

    // Children, in append order (`:66-133`).
    // The 59x58 `UICharacter`: the `PlayerBackground` sheet whole
    // (`UICharacter.cs:100-103`), then the player sprite
    // (`:104-119`'s `Main.PlayerRenderer.DrawPlayer` half — BUG-02): the
    // same `draw_player_preview` the creation screen uses, at the card's
    // `characterScale = 1` (`UICharacterListItem.cs:170`) with the static
    // animation row 0 (`UICharacter.cs:89`) and the entry's `.plr` colours.
    if let Some(sheet) = lookups.ui_player_background {
        push_whole(r, lookups, &sheet, c.preview, [1.0; 4]);
    }
    let anchor = [
        c.preview[0] + c.preview[2] * 0.5 - 10.0,
        c.preview[1] + c.preview[3] * 0.5 - 21.0,
    ];
    crate::render::entities::draw_player_preview(
        r,
        player_assets,
        anchor,
        1.0,
        0,
        &entry.appearance.preview_colors(),
    );
    // The buttons (`:71-121`), each `VAlign 1` except delete's `HAlign 1` —
    // sheets resolve through [`button_sheet`], never a raw ordinal.
    let play_hover = hovered && in_rect(mouse, c.play);
    let fav_hover = hovered && in_rect(mouse, c.favorite);
    let rename_hover = hovered && in_rect(mouse, c.rename);
    let delete_hover = hovered && in_rect(mouse, c.delete);
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_PLAY, entry.is_favorite)],
        c.play,
        play_hover,
        0.4,
    );
    // The favourite sheet flips with the flag, idle 0.8 when favourited (`:80`, `:86`).
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_FAVORITE, entry.is_favorite)],
        c.favorite,
        fav_hover,
        if entry.is_favorite { 0.8 } else { 0.4 },
    );
    // Rename is ctor ordinal 2 but sheet 3 — the favourite's second sheet
    // shifts everything after it ([`button_sheet`], Task 3b review fix 1).
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_RENAME, entry.is_favorite)],
        c.rename,
        rename_hover,
        0.4,
    );
    // The delete button draws (and labels) even while favourited — only its
    // click is dead (`:114-119`, Review Focus 4).
    draw_card_button(
        r,
        lookups,
        lookups.card_buttons[button_sheet(BUTTON_DELETE, entry.is_favorite)],
        c.delete,
        delete_hover,
        0.4,
    );

    // The hover labels (`:140-194`): the left label for play/favorite/rename,
    // the right-aligned one for delete. `UIText.DrawSelf` (`UIText.cs:136-166`)
    // shifts the text up `2*scale` from the element rect.
    let hovered_button = if play_hover {
        Some(BUTTON_PLAY)
    } else if fav_hover {
        Some(BUTTON_FAVORITE)
    } else if rename_hover {
        Some(BUTTON_RENAME)
    } else if delete_hover {
        Some(BUTTON_DELETE)
    } else {
        None
    };
    if let Some(button) = hovered_button {
        let text = button_label(button, entry.is_favorite);
        if button == BUTTON_DELETE {
            // `Left -30, HAlign 1` (`:128-133`): the right edge anchors, the
            // text grows leftward.
            let w = hud_text_width(r, lookups, &text, 1.0);
            border_string(
                r,
                lookups,
                c.delete_label[0] - w,
                c.delete_label[1] - 2.0,
                [1.0; 4],
                &text,
            );
        } else {
            border_string(r, lookups, c.label[0], c.label[1] - 2.0, [1.0; 4], &text);
        }
    }
}

/// The whole screen in the source's z-order: the container panel's `UIPanel`
/// slice, the cards (the list's children), the attached scrollbar, then the
/// root's header / Back / New. `header_w` is the measured `Select Player` width
/// (Death font, unscaled) — the same input [`layout`] took, so the header text
/// centres on the rect the layout produced.
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    player_assets: &crate::render::entities::PlayerAssets,
    screen: Vec2,
    header_w: f32,
    players: &[save_list::PlayerEntry],
    scroll: f32,
    mouse: Vec2,
) {
    let l = resolved_layout(screen, header_w, players.len()).with_scroll(scroll, players.len());
    // `_containerPanel`: bg `(33, 43, 79) * 0.8` (`:46`), border the `UIPanel`
    // default black (`UIPanel.cs:18`).
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(CONTAINER_PANEL_BG));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, l.panel, color::tint(uistate::TEXT_PANEL_BORDER));

    // The cards, in list order, inside the `UIList`'s `OverflowHidden` scissor
    // (`UIElement.cs:222-237`, `UIList.cs:59`): a row straddling the list's
    // bottom edge is painted only down to that edge, exactly as the C# device
    // scissor clips it. The scissor is the list's OUTER rect, not the inner one:
    // `GetClippingRectangle` reads `_innerDimensions` (`:254`), which for the
    // zero-padding `UIList` is the same rect.
    r.push_clip(l.list);
    for (i, entry) in players.iter().enumerate() {
        draw_card(r, lookups, player_assets, &l, i, entry, mouse);
    }
    r.pop_clip();
    // The scrollbar draws only while attached (a `Recalculate`-appended panel
    // child, `:100`), the view over the content height `UIList.UpdateScrollbar`
    // fed it (`UIList.cs:130-136`).
    let content_h = list_content_height(players.len());
    if scrollbar_attaches(l.list[3], content_h) {
        // `UIScrollbar.DrawSelf` (`:103-124`) draws the handle at the live
        // `_viewPosition` — the same value `UIList.cs:181` lifts the rows by.
        panel::draw_scrollbar(r, lookups, l.scrollbar, l.list[3], content_h, l.scroll, false);
    }

    // The header never hovers — no `OnMouseOver` is subscribed (`:59-64`).
    uistate::draw_text_panel(
        r,
        lookups,
        l.header,
        &header_style(),
        &select_player(),
        header_w,
        false,
    );
    // Back/New: `FadedMouseOver`/`FadedMouseOut` (`:70-71`, `:79-80`) — the
    // hover pair `uistate::draw_text_panel` swaps, the idle pair the style
    // carries.
    uistate::draw_text_panel(
        r,
        lookups,
        l.back,
        &fat_button_style(),
        &back(),
        large_text_width(r, lookups, &back()),
        in_rect(mouse, l.back),
    );
    uistate::draw_text_panel(
        r,
        lookups,
        l.new,
        &fat_button_style(),
        &new_button(),
        large_text_width(r, lookups, &new_button()),
        in_rect(mouse, l.new),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn center(r: [f32; 4]) -> Vec2 {
        Vec2::new(r[0] + r[2] * 0.5, r[1] + r[3] * 0.5)
    }

    // Expected values hand-derived from the C# (`UICharacterSelect.cs:35-157`,
    // `UICharacterListItem.cs:49-380`, `UICharacter.cs:34-130`,
    // `UITextPanel.cs:126-134`, `UIText.cs:175-189`, `UIList.cs:120-136`,
    // `UIScrollbar.cs:49-82`) — never from this code.
    const SCREEN: Vec2 = Vec2::new(1600.0, 900.0);
    const LIST: [f32; 4] = [487.0, 232.0, 626.0, 546.0];

    #[test]
    fn layout_matches_the_source_table_at_1600x900() {
        // `layout(Vec2::new(1600, 900), 100.0, 626.0)` — the plan's oracle.
        let l = layout(SCREEN, 100.0, 626.0);
        // Root: Width 0.8*1600 = 1280 -> MaxWidth 650, HAlign .5 -> x 475;
        // Height -220 + 900 = 680, Top 220 (`UICharacterSelect.cs:38-42`).
        assert_eq!(l.root, [475.0, 220.0, 650.0, 680.0]);
        // Panel: Height -110 + 680 = 570 (`:44-45`).
        assert_eq!(l.panel, [475.0, 220.0, 650.0, 570.0]);
        // List: the panel inner (padding 12), 626 wide detached (`:50-52`).
        assert_eq!(l.list, [487.0, 232.0, 626.0, 546.0]);
        // Scrollbar attach rect: HAlign 1 of the panel inner, 20 wide (`:54-57`).
        assert_eq!(l.scrollbar, [1093.0, 232.0, 20.0, 546.0]);
        // Header: MinWidth-sized — 100*0.8 + 30 = 110, centred on 800; height
        // 32*0.8 + 30 = 55.6 (`:59-64`, `UITextPanel.cs:132-134`). Not a
        // 650-wide hint rect.
        assert_eq!(l.header, [745.0, 180.0, 110.0, 55.6]);
        // Back/New: Width 0.5*650 - 10 = 315, H 50, bottom of root - 45
        // (`:65-84`).
        assert_eq!(l.back, [475.0, 805.0, 315.0, 50.0]);
        assert_eq!(l.new, [810.0, 805.0, 315.0, 50.0]);
        // The detached width is screen-derived: 650 - 24.
        assert_eq!(detached_list_width(SCREEN), 626.0);
    }

    #[test]
    fn header_grows_from_the_measured_text() {
        // A wider measurement moves both the MinWidth size and the HAlign-0.5
        // centre (`:59-64`).
        let l = layout(SCREEN, 200.0, 626.0);
        assert_eq!(l.header, [475.0 + (650.0 - 190.0) * 0.5, 180.0, 190.0, 55.6]);
    }

    #[test]
    fn scrollbar_branch_attaches_at_six_cards() {
        // Review Focus 3: the attach/detach branch is ported and unit-tested
        // on a synthetic tall list, not deleted.
        let l = layout(SCREEN, 100.0, detached_list_width(SCREEN));
        assert_eq!(l.list[2], 626.0);
        // The `UIList` content heights (`UIList.cs:120-128`).
        assert_eq!(list_content_height(0), 0.0);
        assert_eq!(list_content_height(1), 96.0);
        assert_eq!(list_content_height(5), 505.0);
        assert_eq!(list_content_height(6), 606.0);
        // 5 cards: 505 < 546 — `SetView` clamps the view onto the content, so
        // `CanScroll` is false (`UIScrollbar.cs:78`, `:49`) — it never attaches
        // below 6 cards.
        assert!(!scrollbar_attaches(546.0, list_content_height(5)));
        // 6 cards: 606 > 546 — attached.
        assert!(scrollbar_attaches(546.0, list_content_height(6)));
        // The empty list clamps to 0 == 0 (`SetView(546, 0)`).
        assert!(!scrollbar_attaches(546.0, list_content_height(0)));
        // The plan's synthetic 700-tall list.
        assert!(scrollbar_attaches(546.0, 700.0));
        // Attached: `Width.Set(-25f, 1f)` -> 601, and the scrollbar rect stands
        // ready at the panel's right edge (`:100-102`).
        let attached = attach_scrollbar(l, true);
        assert_eq!(attached.list, [487.0, 232.0, 601.0, 546.0]);
        assert_eq!(attached.scrollbar, [1093.0, 232.0, 20.0, 546.0]);
        // Detached: `Width.Set(0f, 1f)` -> 626 (`:92-96`).
        assert_eq!(attach_scrollbar(l, false).list[2], 626.0);
        // `resolved_layout` applies the branch from the card count.
        assert_eq!(resolved_layout(SCREEN, 100.0, 5).list[2], 626.0);
        assert_eq!(resolved_layout(SCREEN, 100.0, 6).list[2], 601.0);
    }

    #[test]
    fn dw_2_3_a_double_click_on_a_card_body_yields_double_click_play() {
        // `UICharacterListItem.cs:68-69`: `OnLeftDoubleClick += PlayGame`. The
        // single-click press of the same body stays a bare selection.
        let (sw, _sh) = (1600.0_f32, 900.0_f32);
        let l = resolved_layout(
            Vec2::new(sw, 900.0),
            100.0,
            7,
        );
        let favorites = [false; 7];
        let c0 = card_layout(l.list, 0);
        assert_eq!(
            card_hit(&l, &favorites, center(c0.card), false),
            (Some(0), None)
        );
        assert_eq!(
            card_hit(&l, &favorites, center(c0.card), true),
            (Some(0), Some(CardAction::DoubleClickPlay(0)))
        );
    }

    #[test]
    fn dw_2_3_the_fat_buttons_do_not_double_click_into_play() {
        // The double-click arm is a card-body handler; a second press on `New`
        // or `Back` stays that button's own action.
        let l = resolved_layout(Vec2::new(1600.0, 900.0), 100.0, 7);
        let favorites = [false; 7];
        assert_eq!(
            card_hit(&l, &favorites, center(l.new), true),
            (None, Some(CardAction::New))
        );
        assert_eq!(
            card_hit(&l, &favorites, center(l.back), true),
            (None, Some(CardAction::Back))
        );
    }

    #[test]
    fn card_layout_matches_the_source_rows() {
        // `card_layout([487, 232, 626, 96], 0)` — the plan's one-arg oracle as
        // the index-0 call.
        let c = card_layout([487.0, 232.0, 626.0, 96.0], 0);
        // The card + its 6 px inner (`:61-63`).
        assert_eq!(c.card, [487.0, 232.0, 626.0, 96.0]);
        assert_eq!(uistate::inner(c.card, CARD_PADDING), [493.0, 238.0, 614.0, 84.0]);
        // The 59x58 preview at Left 4 (`:66-67`).
        assert_eq!(c.preview, [497.0, 238.0, 59.0, 58.0]);
        // The buttons: VAlign 1 -> bottom of the inner 322 - 22; pitch 24, no
        // cloud so rename takes 52 (`:71-108`); delete HAlign 1 (`:109-113`).
        assert_eq!(c.play, [497.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.favorite, [521.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.rename, [545.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.delete, [1085.0, 300.0, 22.0, 22.0]);
        // The labels: Left 80 (4+24+24+24+4, `:71-122`), Top -3 + VAlign 1 with
        // the 16-tall empty `UIText` (`UIText.cs:186-189`); the delete label's
        // right edge anchors 30 px in (`:128-133`).
        assert_eq!(c.label, [573.0, 303.0, 0.0, 16.0]);
        assert_eq!(c.delete_label, [1077.0, 303.0, 0.0, 16.0]);
        // DrawSelf extras (`:317-380`).
        assert_eq!(c.name_pos, [562.0, 236.0]); // preview right 556 + 6 / 238 - 2
        assert_eq!(c.divider, [556.0, 259.0, 557.0, 4.0]); // to the OUTER right 1113
        assert_eq!(c.life_panel, [562.0, 267.0, 200.0, 27.0]);
        assert_eq!(c.heart, [567.0, 269.0, 22.0, 22.0]);
        assert_eq!(c.life_text, [594.0, 270.0]);
        assert_eq!(c.mana_icon, [664.0, 269.0, 22.0, 24.0]);
        assert_eq!(c.mana_text, [693.0, 270.0]);
        assert_eq!(c.difficulty_panel, [767.0, 267.0, 140.0, 27.0]);
        assert_eq!(c.playtime_panel, [912.0, 267.0, 195.0, 27.0]); // inner right 1107 - 912
    }

    #[test]
    fn card_layout_row_3_shifts_every_rect() {
        let a = card_layout(LIST, 0);
        let b = card_layout(LIST, 3);
        let dy = 3.0 * CARD_PITCH; // 3*101 = 303
        assert_eq!(b.card, [a.card[0], a.card[1] + dy, a.card[2], a.card[3]]);
        assert_eq!(b.preview, [a.preview[0], a.preview[1] + dy, 59.0, 58.0]);
        assert_eq!(b.play, [a.play[0], a.play[1] + dy, 22.0, 22.0]);
        assert_eq!(b.favorite, [a.favorite[0], a.favorite[1] + dy, 22.0, 22.0]);
        assert_eq!(b.rename, [a.rename[0], a.rename[1] + dy, 22.0, 22.0]);
        assert_eq!(b.delete, [a.delete[0], a.delete[1] + dy, 22.0, 22.0]);
        assert_eq!(b.label, [a.label[0], a.label[1] + dy, 0.0, 16.0]);
        assert_eq!(b.delete_label, [a.delete_label[0], a.delete_label[1] + dy, 0.0, 16.0]);
        assert_eq!(b.name_pos, [a.name_pos[0], a.name_pos[1] + dy]);
        assert_eq!(b.divider, [a.divider[0], a.divider[1] + dy, a.divider[2], 4.0]);
        assert_eq!(b.life_panel, [a.life_panel[0], a.life_panel[1] + dy, 200.0, 27.0]);
        assert_eq!(b.heart, [a.heart[0], a.heart[1] + dy, 22.0, 22.0]);
        assert_eq!(b.life_text, [a.life_text[0], a.life_text[1] + dy]);
        assert_eq!(b.mana_icon, [a.mana_icon[0], a.mana_icon[1] + dy, 22.0, 24.0]);
        assert_eq!(b.mana_text, [a.mana_text[0], a.mana_text[1] + dy]);
        assert_eq!(
            b.difficulty_panel,
            [a.difficulty_panel[0], a.difficulty_panel[1] + dy, 140.0, 27.0]
        );
        assert_eq!(
            b.playtime_panel,
            [a.playtime_panel[0], a.playtime_panel[1] + dy, a.playtime_panel[2], 27.0]
        );
        // Absolute anchor: row 3's card top is 232 + 303.
        assert_eq!(b.card, [487.0, 535.0, 626.0, 96.0]);
    }

    #[test]
    fn card_layout_follows_the_attached_list_width() {
        // Review Focus 3: the scrollbar changes the LIST width, and the card
        // widths (so the HAlign-1 delete and the divider) follow it.
        let l = resolved_layout(SCREEN, 100.0, 6); // attached
        assert_eq!(l.list[2], 601.0);
        let c = card_layout(l.list, 0);
        assert_eq!(c.card, [487.0, 232.0, 601.0, 96.0]);
        assert_eq!(c.delete, [487.0 + 601.0 - 6.0 - 22.0, 300.0, 22.0, 22.0]);
        assert_eq!(c.divider[2], 487.0 + 601.0 - 556.0);
    }

    #[test]
    fn inner_panel_rects_are_the_three_piece_draw_panel() {
        // `UICharacterListItem.cs:310-315`: 8 px edges, the middle stretched to
        // width - 16, all pieces the sheet's full height at draw time. NOT the
        // 9-slice `Utils.DrawSplicedPanel`.
        let rects = inner_panel_rects(562.0, 267.0, 200.0);
        assert_eq!(rects[0], [562.0, 267.0, 8.0, 0.0]);
        assert_eq!(rects[1], [570.0, 267.0, 184.0, 0.0]);
        assert_eq!(rects[2], [754.0, 267.0, 8.0, 0.0]);
    }

    #[test]
    fn card_colors_match_the_xna_truncations() {
        // MouseOut (`:302-308`): `(63,82,151)*0.7` and `(89,116,213)*0.7`,
        // XNA byte truncation — `(44,57,105,178)` and `(62,81,149,178)` (213*0.7
        // = 149.1 -> 149).
        assert_eq!(card_colors(false), ([44, 57, 105, 178], [62, 81, 149, 178]));
        assert_eq!([44, 57, 105, 178], color::mul([63, 82, 151, 255], 0.7));
        assert_eq!([62, 81, 149, 178], color::mul([89, 116, 213, 255], 0.7));
        // MouseOver (`:294-300`): `(73,94,171)` / `(89,116,213)`, opaque.
        assert_eq!(card_colors(true), ([73, 94, 171, 255], [89, 116, 213, 255]));
    }

    #[test]
    fn difficulty_texts_follow_the_switch() {
        // `:353-370` + `Main.cs:66765-66768`.
        assert_eq!(difficulty_text(0), (crate::lang::text("UI.Softcore"), [255, 255, 255, 255]));
        assert_eq!(difficulty_text(1), (crate::lang::text("UI.Mediumcore"), uistate::MC_COLOR));
        assert_eq!(difficulty_text(2), (crate::lang::text("UI.Hardcore"), uistate::HC_COLOR));
        assert_eq!(difficulty_text(3), (crate::lang::text("UI.Creative"), uistate::CREATIVE_MODE_COLOR));
        // The C# switch has no default arm — an unknown difficulty falls through
        // with an empty label; the port lands it on Classic.
        assert_eq!(difficulty_text(9).0, crate::lang::text("UI.Softcore"));
    }

    #[test]
    fn button_labels_match_the_hover_table() {
        // `:140-194`.
        assert_eq!(button_label(BUTTON_PLAY, false), "Play");
        assert_eq!(button_label(BUTTON_FAVORITE, false), "Favorite");
        assert_eq!(button_label(BUTTON_FAVORITE, true), "Unfavorite");
        assert_eq!(button_label(BUTTON_RENAME, false), "Rename");
        assert_eq!(button_label(BUTTON_DELETE, false), "Delete");
        assert_eq!(button_label(BUTTON_DELETE, true), "Cannot Delete (Favorited)");
    }

    #[test]
    fn button_sheets_map_the_ctor_ordinals_to_the_sheet_array() {
        // The BUTTON_* ordinals are the ctor's append order (`:71-121`) — the
        // label table's key — while `Lookups::card_buttons` is
        // `[Play, FavoriteActive, FavoriteInactive, Rename, Delete]`
        // (assets.rs:956-963): the favourite's second sheet shifts Rename and
        // Delete one later, so indexing with a raw `BUTTON_RENAME` ordinal
        // drew the `FavoriteInactive` sprite (Task 3b review fix 1). Only the
        // favourite's own sheet flips with the flag — the others ignore it.
        assert_eq!(button_sheet(BUTTON_PLAY, true), 0);
        assert_eq!(button_sheet(BUTTON_RENAME, true), 3);
        assert_eq!(button_sheet(BUTTON_DELETE, true), 4);
        // The primary mapping, sheet names per `assets.rs:956-963`:
        assert_eq!(button_sheet(BUTTON_PLAY, false), 0); // ButtonPlay
        assert_eq!(button_sheet(BUTTON_FAVORITE, true), 1); // ButtonFavoriteActive
        assert_eq!(button_sheet(BUTTON_FAVORITE, false), 2); // ButtonFavoriteInactive
        assert_eq!(button_sheet(BUTTON_RENAME, false), 3); // ButtonRename — not 2!
        assert_eq!(button_sheet(BUTTON_DELETE, false), 4); // ButtonDelete
    }

    #[test]
    fn card_hit_routes_the_button_centres() {
        let l = resolved_layout(SCREEN, 100.0, 2); // 2 cards -> detached, 626 wide
        let center = |rect: [f32; 4]| Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
        let c0 = card_layout(l.list, 0);
        let c1 = card_layout(l.list, 1);
        let favorites = [false, true];
        assert_eq!(
            card_hit(&l, &favorites, center(c0.play), false),
            (Some(0), Some(CardAction::Play(0)))
        );
        assert_eq!(
            card_hit(&l, &favorites, center(c0.favorite), false),
            (Some(0), Some(CardAction::ToggleFavorite(0)))
        );
        assert_eq!(
            card_hit(&l, &favorites, center(c0.rename), false),
            (Some(0), Some(CardAction::Rename(0)))
        );
        // Card 0 is not favourited -> the delete click routes to the confirm.
        assert_eq!(
            card_hit(&l, &favorites, center(c0.delete), false),
            (Some(0), Some(CardAction::Delete(0)))
        );
        // Card 1 IS favourited -> `DeleteBlocked`, not `Delete` (Review Focus 4:
        // `:114-117` never subscribes the handler).
        assert_eq!(
            card_hit(&l, &favorites, center(c1.delete), false),
            (Some(1), Some(CardAction::DeleteBlocked))
        );
        // A card body away from every button: hovered, but no action.
        let body = Vec2::new(c0.name_pos[0] + 30.0, c0.card[1] + 8.0);
        assert_eq!(card_hit(&l, &favorites, body, false), (Some(0), None));
        // The fat buttons.
        assert_eq!(
            card_hit(&l, &favorites, center(l.back), false),
            (None, Some(CardAction::Back))
        );
        assert_eq!(
            card_hit(&l, &favorites, center(l.new), false),
            (None, Some(CardAction::New))
        );
        // Off-screen: nothing.
        assert_eq!(card_hit(&l, &favorites, Vec2::new(10.0, 10.0), false), (None, None));
        // The flat form.
        assert_eq!(hit(&l, &favorites, center(c0.play)), Some(CardAction::Play(0)));
        assert_eq!(hit(&l, &favorites, Vec2::new(10.0, 10.0)), None);
    }

    #[test]
    fn back_and_new_beat_the_cards_in_the_overlap_zone() {
        // `GetElementAt`'s reverse-append priority (`UIElement.cs:324-334`)
        // is observable: with 6 players the scrollbar attaches and the last
        // card's rows run under the fat buttons — card 5 spans y 737-833
        // while Back/New sit at y 805-855 — and a click at y >= 805 in the
        // overlap is Back's/New's, never the card's (Task 3b review fix 2).
        let l = resolved_layout(SCREEN, 100.0, 6); // attached
        let c5 = card_layout(l.list, 5);
        assert_eq!(c5.card, [487.0, 737.0, 601.0, 96.0]); // y 737-833
        assert_eq!(l.back, [475.0, 805.0, 315.0, 50.0]); // y 805-855
        assert_eq!(l.new, [810.0, 805.0, 315.0, 50.0]);
        let favorites = [false; 6];
        // Card 5's rename button and Back share (550, 815) — Back wins.
        let p = Vec2::new(550.0, 815.0);
        assert!(in_rect(p, c5.rename));
        assert!(in_rect(p, l.back));
        assert_eq!(card_hit(&l, &favorites, p, false), (None, Some(CardAction::Back)));
        // Card 5's delete button and New share (1070, 815) — New wins.
        let p = Vec2::new(1070.0, 815.0);
        assert!(in_rect(p, c5.delete));
        assert!(in_rect(p, l.new));
        assert_eq!(card_hit(&l, &favorites, p, false), (None, Some(CardAction::New)));
        // Just above the fat buttons but past the list's bottom edge (778):
        // card 5's rect still covers y 800, yet the C# cannot reach it —
        // `GetElementAt` filters every child by `ContainsPoint` on the way down
        // (`UIElement.cs:327-333`), and the click lands outside the `UIList`'s
        // own rect, so the list is skipped and the point falls through to
        // nothing. The draw agrees: `OverflowHidden` clips card 5 at 778
        // (`:222-237`).
        assert_eq!(
            card_hit(&l, &favorites, Vec2::new(550.0, 800.0), false),
            (None, None)
        );
        // One pixel above the edge the same card is hit.
        assert_eq!(
            card_hit(&l, &favorites, Vec2::new(550.0, 777.0), false),
            (Some(5), None)
        );
    }

    #[test]
    fn clicks_below_the_list_bottom_never_reach_a_card() {
        // The `OverflowHidden` break gate (Task 3/4 review fix): with 6
        // players the list scrolls, card 6 would start at y 838 — past the
        // list bottom 778 — and the draw never paints it, so the hit-test
        // must not answer for it either. (550, 860) is inside that phantom
        // row and below Back's hit box (y 805-855).
        let l = resolved_layout(SCREEN, 100.0, 6); // attached
        assert_eq!(l.list[1] + l.list[3], 778.0);
        assert_eq!(card_layout(l.list, 6).card[1], 838.0, "the phantom row's top");
        assert!(!in_rect(Vec2::new(550.0, 860.0), l.back), "below Back too");
        assert_eq!(card_hit(&l, &[false; 7], Vec2::new(550.0, 860.0), false), (None, None));
        // Card 5 straddles the bottom (top 737 < 778) and stays hit — the
        // same rows the draw still paints.
        let favorites = [false; 6];
        assert_eq!(
            card_hit(&l, &favorites, Vec2::new(550.0, 777.0), false),
            (Some(5), None)
        );
    }

    #[test]
    fn sort_players_is_the_save_list_implementation() {
        // T2 owns the `UpdatePlayersList` ordering; the re-export keeps one copy
        // instead of duplicating the chain.
        let mut players = vec![
            save_list::PlayerEntry {
                name: "B".into(),
                difficulty: 0,
                life_max: 100,
                mana_max: 20,
                is_favorite: false,
                playable: true,
                deletable: true,
                path: None,
                appearance: save_list::Appearance::default(),
                inv_favorites: 0,
            },
            save_list::PlayerEntry {
                name: "A".into(),
                difficulty: 0,
                life_max: 100,
                mana_max: 20,
                is_favorite: true,
                playable: true,
                deletable: true,
                path: None,
                appearance: save_list::Appearance::default(),
                inv_favorites: 0,
            },
        ];
        sort_players(&mut players);
        assert_eq!(
            players.iter().map(|p| p.name.as_str()).collect::<Vec<_>>(),
            vec!["A", "B"]
        );
    }

    // --- WAVE2/F1: the list's scroll offset (`UIList._innerList.Top`) ----------
    //
    // Every expected number below is the A6 geometry harness's (`_a6_geom.py`,
    // `UI-INPUT-RUST` §5.3) or hand-derived from `UIList.cs`/`UIScrollbar.cs`:
    // list rects 1280x720 [327,232,626,366], 1600x900 [487,232,626,546],
    // 1920x1080 [647,232,626,726]; a card is 96 tall on a 101 px pitch; the
    // attached list is 25 px narrower (`UICharacterSelect.cs:102`).

    #[test]
    fn f1_max_scroll_is_the_content_height_minus_the_view() {
        // `UIScrollbar.SetView` clamps the position into `[0, maxViewSize -
        // viewSize]` (`UIScrollbar.cs:75`), with `viewSize` the `UIList`'s inner
        // height and `maxViewSize` its `_innerListHeight` (`UIList.cs:130-137`).
        assert_eq!(
            resolved_layout(Vec2::new(1280.0, 720.0), 100.0, 4).list,
            [327.0, 232.0, 601.0, 366.0] // attached: 626 - 25
        );
        assert_eq!(
            resolved_layout(SCREEN, 100.0, 6).list,
            [487.0, 232.0, 601.0, 546.0]
        );
        assert_eq!(
            resolved_layout(Vec2::new(1920.0, 1080.0), 100.0, 8).list,
            [647.0, 232.0, 601.0, 726.0]
        );
        // 4 cards: 404 px of rows in a 366 px view -> 38 px of travel.
        assert_eq!(list_content_height(4), 404.0);
        assert_eq!(max_scroll(366.0, list_content_height(4)), 38.0);
        // 6 cards at 1600x900: 606 - 546.
        assert_eq!(max_scroll(546.0, list_content_height(6)), 60.0);
        // 8 cards at 1920x1080: 808 - 726.
        assert_eq!(max_scroll(726.0, list_content_height(8)), 82.0);
        // A list shorter than its view cannot scroll (`CanScroll`, `:49`):
        // 5 cards (505 px) and a single card (96 px) both sit at 0.
        assert_eq!(max_scroll(546.0, list_content_height(5)), 0.0);
        assert_eq!(max_scroll(546.0, list_content_height(1)), 0.0);
        // The clamp itself is `ViewPosition`'s setter (`UIScrollbar.cs:43-46`).
        assert_eq!(clamp_scroll(500.0, 546.0, 606.0), 60.0);
        assert_eq!(clamp_scroll(-40.0, 546.0, 606.0), 0.0);
    }

    #[test]
    fn f1_the_wheel_moves_the_view_120_px_per_notch_and_clamps() {
        // `UIList.ScrollWheel` (`UIList.cs:107-114`) spends the C# wheel value
        // 1:1 on `ViewPosition` (`:112`), and that value is XNA's raw counter —
        // 120 units per detent (`PlayerInput.cs:898-899` copies
        // `ScrollWheelValue - ScrollWheelValueOld` into `ScrollWheelDeltaForUI`).
        assert_eq!(WHEEL_STEP, 120.0);
        // One notch down from the top of the 6-card 1600x900 list: 120 px is
        // clamped to the 60 px range (`UIScrollbar.cs:45`).
        assert_eq!(scroll_wheel(0.0, -1, 546.0, 606.0), 60.0);
        // One notch up from the bottom returns to the top.
        assert_eq!(scroll_wheel(60.0, 1, 546.0, 606.0), 0.0);
        // 8 cards at 1280x720 (366 px view over 808 px of rows): 120, then
        // 240, then 400 clamped to the 442 px range.
        assert_eq!(scroll_wheel(0.0, -1, 366.0, 808.0), 120.0);
        assert_eq!(scroll_wheel(120.0, -1, 366.0, 808.0), 240.0);
        assert_eq!(scroll_wheel(400.0, -1, 366.0, 808.0), 442.0);
        // Nothing to scroll: the wheel is inert.
        assert_eq!(scroll_wheel(0.0, -1, 546.0, 505.0), 0.0);
    }

    #[test]
    fn f1_the_last_cards_delete_hits_delete_not_new() {
        // A6: at 1600x900 with 6 cards the list is 546 tall while the content is
        // 606, so card 5 spans y 737-833 and its `ButtonDelete` sits at
        // y 805-827 — below the list bottom 778. `GetElementAt` cannot descend
        // into it (`framework.rs:128`'s viewport, `UIElement.cs:327-341`) and the
        // point lands on the fat `New` button (y 805-855): the A6 harness
        // measured `hit_id=1`, so Delete opened the creation screen.
        let l = resolved_layout(SCREEN, 100.0, 6);
        assert_eq!(l.new, [810.0, 805.0, 315.0, 50.0]);
        assert_eq!(card_layout(l.list, 5).delete, [1060.0, 805.0, 22.0, 22.0]);
        assert_eq!(l.scroll, 0.0, "a fresh screen starts at the top");
        let favorites = [false; 6];
        assert_eq!(
            card_hit(&l, &favorites, Vec2::new(1071.0, 816.0), false),
            (None, Some(CardAction::New))
        );
        // Scrolled to the bottom `UIList.cs:181` lifts the rows by the scroll
        // value, 606 - 546 = 60 px: the same button sits at y 745-767, inside
        // the view (bottom 778), and the click is the card's own handler
        // (`UICharacterListItem.cs:242-254`).
        let l = l.with_scroll(60.0, 6);
        assert_eq!(l.scroll, 60.0);
        assert_eq!(l.inner_list(), [487.0, 172.0, 601.0, 546.0]);
        assert_eq!(
            card_layout(l.inner_list(), 5).delete,
            [1060.0, 745.0, 22.0, 22.0]
        );
        assert_eq!(
            card_hit(&l, &favorites, Vec2::new(1071.0, 756.0), false),
            (Some(5), Some(CardAction::Delete(5)))
        );
    }

    #[test]
    fn f1_every_resolution_reaches_the_last_cards_delete_after_scrolling() {
        // The A6 table, one row per resolution: with no scroll the last card's
        // Delete is below the view (or, where the fat buttons overlap, lands on
        // `New`); at the maximum scroll it is inside the view and routes to
        // `Delete`. `delete_x` is the attached list's HAlign-1 column
        // (`ix + iw - 22`, `UICharacterListItem.cs:109-113`).
        struct Case {
            screen: Vec2,
            cards: usize,
            max: f32,
            delete_x: f32,
            blocked_y: f32,
            blocked: (Option<usize>, Option<CardAction>),
            scrolled_y: f32,
        }
        let cases = [
            // 1280x720, 4 cards: delete y 603-625 vs the 598 px view bottom —
            // a dead click (A6 `hit_id=None`).
            Case {
                screen: Vec2::new(1280.0, 720.0),
                cards: 4,
                max: 38.0,
                delete_x: 911.0,
                blocked_y: 614.0,
                blocked: (None, None),
                scrolled_y: 576.0,
            },
            // 1600x900, 6 cards: delete y 805-827, below 778 — the fat `New`
            // button's row 805-855 takes it (A6 `hit_id=1`).
            Case {
                screen: SCREEN,
                cards: 6,
                max: 60.0,
                delete_x: 1071.0,
                blocked_y: 816.0,
                blocked: (None, Some(CardAction::New)),
                scrolled_y: 756.0,
            },
            // 1920x1080, 8 cards: delete y 1007-1029 vs the 958 px view — the
            // fat `New` button (y 985-1035) takes it (A6 `hit_id=1`).
            Case {
                screen: Vec2::new(1920.0, 1080.0),
                cards: 8,
                max: 82.0,
                delete_x: 1231.0,
                blocked_y: 1018.0,
                blocked: (None, Some(CardAction::New)),
                scrolled_y: 936.0,
            },
        ];
        for case in cases {
            let favorites = vec![false; case.cards];
            let last = case.cards - 1;
            let l = resolved_layout(case.screen, 100.0, case.cards);
            assert_eq!(
                max_scroll(l.list[3], list_content_height(case.cards)),
                case.max,
                "{:?} {} cards",
                case.screen,
                case.cards
            );
            // Card i's Delete row: 300 + 101*i - scroll .. +22 (`list.y` 232,
            // card padding 6, card 96 tall with VAlign 1 22 px buttons).
            let blocked = Vec2::new(case.delete_x, case.blocked_y);
            assert_eq!(
                card_hit(&l, &favorites, blocked, false),
                case.blocked,
                "{:?} {} cards, unscrolled",
                case.screen,
                case.cards
            );
            let l = l.with_scroll(case.max, case.cards);
            let delete = card_layout(l.inner_list(), last).delete;
            assert_eq!(delete[0], case.delete_x - 11.0);
            assert_eq!(delete[1], case.scrolled_y - 11.0);
            assert_eq!(
                card_hit(
                    &l,
                    &favorites,
                    Vec2::new(case.delete_x, case.scrolled_y),
                    false
                ),
                (Some(last), Some(CardAction::Delete(last))),
                "{:?} {} cards, scrolled to {}",
                case.screen,
                case.cards,
                case.max
            );
        }
    }

    #[test]
    fn f1_the_scrollbar_handle_follows_the_written_position() {
        // `UIScrollbar.GetHandleRectangle` (`UIScrollbar.cs:85-94`): the inner
        // rect drops the 5 px `PaddingTop`/`PaddingBottom` (`:68-69`), the
        // handle top is `inner.Y + inner.H * (pos/maxView) - 3` and its height
        // `inner.H * (view/maxView) + 7`. At 1600x900 the bar is
        // [1093,232,20,546] -> inner y 237, height 536; with 8 cards the view is
        // 546 and the max view 808, so the handle is 369 tall and travels
        // 0..262.
        let l = resolved_layout(SCREEN, 100.0, 8); // attached
        assert_eq!(l.scrollbar, [1093.0, 232.0, 20.0, 546.0]);
        assert_eq!(max_scroll(546.0, list_content_height(8)), 262.0);
        let handle = |pos: f32| panel::scrollbar_handle_at(l.scrollbar, 546.0, 808.0, pos);
        assert_eq!(handle(0.0), [1093.0, 234.0, 20.0, 369.0]);
        // 536 * 82/808 = 54.396 -> trunc 291 - 3.
        assert_eq!(handle(82.0), [1093.0, 288.0, 20.0, 369.0]);
        // 536 * 262/808 = 173.80 -> trunc 410 - 3.
        assert_eq!(handle(262.0), [1093.0, 407.0, 20.0, 369.0]);
    }

    #[test]
    fn f1_a_press_on_the_scrollbar_grabs_the_handle_or_jumps_the_track() {
        // `UIScrollbar.LeftMouseDown` (`UIScrollbar.cs:126-143`): a press on the
        // handle latches `_isDragging` with `_dragYOffset = mouse.Y - handle.Y`
        // (`:134-135`); a track press jumps the view to
        // `(mouse.Y - inner.Y - handle.Height/2) / inner.H * maxView`, clamped
        // to `[0, maxView - view]` (`:140`). The held drag re-derives the
        // position every frame (`:109-112`).
        let l = resolved_layout(SCREEN, 100.0, 8); // view 546, content 808, max 262
        let over_bar = |y: f32| Vec2::new(1103.0, y);
        // The handle at the top spans y 234-603, so a press at y 300 grabs it
        // with a 66 px offset and leaves the view where it was.
        assert_eq!(
            scrollbar_press(&l, 8, over_bar(300.0)),
            Some((0.0, Some(66.0)))
        );
        // A press on the track below the handle (y 700 is inside the inner
        // 237..773) jumps to (700 - 237 - 184) / 536 * 808 = 420.6, clamped to
        // the 262 px range.
        assert_eq!(scrollbar_press(&l, 8, over_bar(700.0)), Some((262.0, None)));
        // 20 cards: 2020 px of rows, so the handle is 151 tall (234..385) and
        // y 500 is track: (500 - 237 - 75) / 536 * 2020 = 708.5075, inside the
        // 1474 px range.
        let tall = resolved_layout(SCREEN, 100.0, 20);
        let Some((jump, None)) = scrollbar_press(&tall, 20, over_bar(500.0)) else {
            panic!("the track press must jump, not drag");
        };
        assert!((jump - 708.5075).abs() < 1e-3, "{jump}");
        // The drag frame with the latched offset: y 600 -> 532.1, clamped; y
        // 250 -> (250 - 237 - 10) / 536 * 808 = 4.522.
        assert_eq!(scrollbar_drag(&l, 8, 10.0, 600.0), 262.0);
        let dragged = scrollbar_drag(&l, 8, 10.0, 250.0);
        assert!((dragged - 4.522_39).abs() < 1e-3, "{dragged}");
        // A press off the bar is not the scrollbar's, and a short list has no
        // attached bar at all (`CanScroll`, `UIScrollbar.cs:49`).
        assert_eq!(scrollbar_press(&l, 8, Vec2::new(500.0, 300.0)), None);
        let short = resolved_layout(SCREEN, 100.0, 2);
        assert_eq!(scrollbar_press(&short, 2, over_bar(300.0)), None);
        // The wheel reaches the list over a gap (`UIInnerList.ContainsPoint` is
        // `true`, `UIList.cs:16-19`) and over the attached bar, but not over
        // the header/Back/New outside the list rect.
        assert!(wheel_target(&l, 8, Vec2::new(500.0, 400.0)));
        assert!(wheel_target(&l, 8, over_bar(400.0)));
        assert!(!wheel_target(&l, 8, Vec2::new(500.0, 830.0)));
    }

    // --- W4-d81 batch 7: UICharacterListItem (census row 17) --------------------
    // Written in code-only mode — NOT RUN here. Hand-derived from
    // `UICharacterListItem.cs:208-214`, `:285-292`, `:324-330`, `:376-378`,
    // `ID/StatusID.cs:25-32`.

    #[test]
    fn the_card_order_and_rename_trim_match_the_source() {
        // `CompareTo` (`:285-292`) rides `_orderInList`, the ctor index.
        assert_eq!(compare_card_order(0, 1), std::cmp::Ordering::Less);
        assert_eq!(compare_card_order(3, 3), std::cmp::Ordering::Equal);
        assert_eq!(compare_card_order(5, 2), std::cmp::Ordering::Greater);
        // `OnFinishedSettingName` (`:210`): `name.Trim()` both ends.
        assert_eq!(renamed_player_name("  Bob  "), "Bob");
        assert_eq!(renamed_player_name("\tAlice\n"), "Alice");
        assert_eq!(renamed_player_name(""), "");
    }

    #[test]
    fn the_unplayable_name_prefixes_the_status() {
        // `StatusID` (`ID/StatusID.cs:25-32`): the seven field names.
        assert_eq!(load_status_name(0), Some("Ok"));
        assert_eq!(load_status_name(1), Some("LaterVersion"));
        assert_eq!(load_status_name(3), Some("EmptyFile"));
        assert_eq!(load_status_name(6), Some("BadFooter"));
        assert_eq!(load_status_name(7), None); // GetName would throw
        // `DrawSelf` (`:324-330`): Ok -> plain White; else "(Status) name" Gray.
        assert_eq!(card_name_text("Bob", 0), ("Bob".to_string(), false));
        assert_eq!(
            card_name_text("Bob", 1),
            ("(LaterVersion) Bob".to_string(), true)
        );
        assert_eq!(
            card_name_text("Alice", 3),
            ("(EmptyFile) Alice".to_string(), true)
        );
    }

    #[test]
    fn the_playtime_formatter_rides_days_into_hours() {
        // `:376-378`: num5 = Days*24 + Hours, "0"-padded under 10, then
        // ToString("\:mm\:ss").
        assert_eq!(playtime_text(0), "00:00:00");
        assert_eq!(playtime_text(3_661), "01:01:01");
        assert_eq!(playtime_text(35_999), "09:59:59"); // still padded
        assert_eq!(playtime_text(36_000), "10:00:00"); // pad drops at 10
        // 1 day + 2 h + 3 m + 4 s -> 26:03:04 (`Days*24 + Hours`).
        assert_eq!(playtime_text(86_400 + 7_200 + 180 + 4), "26:03:04");
        // 10 days -> 240 hours, no wrap.
        assert_eq!(playtime_text(864_000), "240:00:00");
    }
}
