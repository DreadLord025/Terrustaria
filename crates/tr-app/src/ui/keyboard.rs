//! The virtual keyboard (`Main.menuMode == 888`'s naming state): a faithful
//! immediate port of `UIVirtualKeyboard`
//! (`GameContent/UI/States/UIVirtualKeyboard.cs`).
//!
//! Same shape as `ui/char_select.rs`: a pure [`layout`] resolving the source's
//! `StyleDimension`s through the shared `uistate::resolve` pass, [`key_rect`]
//! behind both the hit-tests and the draw (Review Focus 2), and a [`draw`]
//! that replays the source's child order. Nothing panics with
//! `Lookups::default()` — every sheet is an `Option` guard (Review Focus 7).
//!
//! **Two layers** (Review Focus 6). `DrawSelf` (`UIVirtualKeyboard.cs:680-746`)
//! keeps `outerLayer1` — the 50-key `UITextPanel` grid inside the cyan panel
//! plus the small 0.4 Submit/Cancel pair — only while
//! `PlayerInput.SettingsForUI.ShowGamepadHints`; without a gamepad that is
//! false, so the *visible* screen is `outerLayer2` (the 0.72 Silver-text
//! Submit/Cancel pair) with the textbox and label shifted down 50 px
//! (`:716-725`). Both layers are ported: the grid is pure geometry
//! ([`key_rect`]), and `show_gamepad_hints` selects it — a parameter the App
//! pins to `false` (`TR_VK_GRID=1` is the debug override for screenshots).
//!
//! Input: the on-screen keys type through [`type_char`] (the `TypeText`
//! handler, `:517-533` — it auto-unlatches a seeded Shift), while the physical
//! keyboard edits the text through [`write_str`] (the `Main.GetInputText` path,
//! `:750-788`, which never touches `KeyState`); `Backspace` pops
//! (`:392-397`); Enter submits only when [`text_valid`] holds
//! (`:358-365`, `:970-978`).
//!
//! Documented deviations (both font-metric, the layout stays font-free — the
//! plan-wide `title_width` rule): the `MinWidth` growth of the textbox and the
//! `UIText` label with the measured string is not ported (the fixed
//! `Width.Pixels` 516 wins — both vanilla labels, `Lang.menu[45]`/`[48]`,
//! fit it), and the `:738-740` `HAlign` switch is dead at this width (it
//! fires only when the box outgrows the 1600 px state root).

use glam::Vec2;
use tr_render::{SheetRect, SpriteRenderer};

use super::color;
use super::panel;
use super::text::font_width;
use super::uistate::{self, Style};
use crate::assets;
use crate::push_font_text;

// --- Source strings (Localization `UI.*` / `LegacyMenu.*`, English) -----------

/// `Language.GetText("UI.Submit")` (`UIVirtualKeyboard.cs:291`, `:327`).
pub const SUBMIT: &str = "Submit";
/// `Language.GetText("UI.Cancel")` (`:314`, `:367`).
pub const CANCEL: &str = "Cancel";
/// `Language.GetText("UI.SpaceButton")` (`:452`).
pub const SPACE: &str = "Space";
/// The symbol toggle's own label (`CreateKeyboardButton("@%", 1, 4, 1)`, `:263`).
pub const SYMBOL_BUTTON_LABEL: &str = "@%";
/// `Lang.menu[45]` — the character-name keyboard's label
/// (`UICharacterListItem.cs:199`, `LegacyMenu.45`).
pub const ENTER_CHARACTER_NAME: &str = "Enter Character Name:";
/// `Lang.menu[48]` — the world-name keyboard's label
/// (`UIWorldListItem.cs:418`, `LegacyMenu.48`).
pub const ENTER_WORLD_NAME: &str = "Enter World Name:";

/// The default key labels, column bands of 10 (`DEFAULT_KEYS`, `:33`): the
/// grid loop reads `DEFAULT_KEYS[j * 10 + i]` (`:223`), so the string is
/// row-major — band 0 `1234567890`, band 1 `qwertyuiop`, band 2 `asdfghjkl'`,
/// band 3 `zxcvbnm,.?`.
const DEFAULT_KEYS: &str = "1234567890qwertyuiopasdfghjkl'zxcvbnm,.?";
/// `SHIFT_KEYS` (`:34`) — the uppercase twin.
const SHIFT_KEYS: &str = "1234567890QWERTYUIOPASDFGHJKL'ZXCVBNM,.?";
/// `SYMBOL_KEYS` (`:36`) — band 1 `!@#$%^&*()`, band 2 `-_+=/\{}[]`,
/// band 3 `<>;:"` + backtick + `|~£¥`.
const SYMBOL_KEYS: &str = "1234567890!@#$%^&*()-_+=/\\{}[]<>;:\"`|~£¥";

// --- The geometry literals (`CreateKeyboardButton`, `:655-673`) ---------------

/// `KEY_WIDTH = 48f` (`:40`).
const KEY_WIDTH: f32 = 48.0;
/// `KEY_HEIGHT = 37f` (`:42`).
const KEY_HEIGHT: f32 = 37.0;
/// `KEY_SPACING = 4f` (`:38`) — the column pitch is `48 + 4 = 52`, the row
/// pitch `37 + 4 = 41`.
const KEY_SPACING: f32 = 4.0;
/// The textbox's scale (`new UITextBox("", 0.78f, large: true)`, `:210`).
const TEXTBOX_SCALE: f32 = 0.78;
/// The label's scale (`new UIText(labelText, 0.75f, large: true)`, `:408`).
const LABEL_SCALE: f32 = 0.75;
/// The on-screen keys'/small pair's scale (`:291`, `:314`, `:658`).
const KEY_TEXT_SCALE: f32 = 0.4;
/// The desktop Submit/Cancel pair's idle scale (`:327`, `:367`).
const SUBMIT2_SCALE: f32 = 0.72;
/// The desktop pair's hover scale (`:336`, `:372`).
const SUBMIT2_HOVER_SCALE: f32 = 0.85;
/// `UIPanel.cs:32` — every keyboard panel/box keeps the 12 px default padding
/// (the keys zero only the sides they zero, cited at the use sites).
const UI_PANEL_PADDING: [f32; 4] = [12.0; 4];
/// The keyboard panel: `BackgroundColor = new Color(23, 33, 69) * 0.7f` (`:206`).
const PANEL_BG: [u8; 4] = [16, 23, 48, 178];
/// `_internalBorderColor = new Color(89, 116, 213)` (`:68`).
const INTERNAL_BORDER_COLOR: [u8; 4] = [89, 116, 213, 255];
/// `Color.Gray` — the disabled Submit text (`:603`).
const GRAY: [u8; 4] = [128, 128, 128, 255];
/// `Color.Silver` — the desktop pair's idle text (`:328`, `:368`).
const SILVER: [u8; 4] = [192, 192, 192, 255];

// --- Model --------------------------------------------------------------------

/// What the keyboard edits — the `UIVirtualKeyboard` ctor args (`:159`).
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    /// The `UIText` label above the box (`:408`): `Lang.menu[45]` /
    /// `Lang.menu[48]` ([`ENTER_CHARACTER_NAME`] / [`ENTER_WORLD_NAME`]).
    pub label: String,
    /// `startingText` — `this.Text = startingText` (`:429`).
    pub initial: String,
    /// `_allowEmpty` (`:162`): the rename keyboards open with
    /// `allowEmpty: true` (`UICharacterListItem.cs:199`,
    /// `UIWorldListItem.cs:419`).
    pub allow_empty: bool,
    /// `SetTextMaxLength(maxLength)` (`:428`; the ctor default is 20, `:159`;
    /// the world rename passes 27, `UIWorldListItem.cs:419`).
    pub max_length: usize,
}

/// The `KeyState` enum (`:19-24`; the port orders the variants Default, Shift,
/// Symbol after the plan, the C# numbers them Default, Symbol, Shift).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyState {
    /// `KeyState.Default` — the lowercase letters.
    Default,
    /// `KeyState.Shift` — the uppercase letters (latched by the shift button).
    Shift,
    /// `KeyState.Symbol` — the symbol bands.
    Symbol,
}

/// The live keyboard state: the `_textBox` string plus the `_keyState` the
/// label tables hang off (`:82`, `:535-583`).
#[derive(Clone, Debug, PartialEq)]
pub struct KeyboardState {
    /// `Text` (`:110-121`) — the `_textBox.Text` the box renders.
    pub text: String,
    /// `_keyState` (`:82`).
    pub key_state: KeyState,
    /// The plan's mirror of `key_state == Shift` (the ctor seeds Shift when
    /// the text is empty, `:430-433`); kept in sync by every mutator here.
    pub shift_latched: bool,
    /// `_maxLength` (`UITextBox.cs:17`) — from the [`Request`].
    pub max_length: usize,
}

impl KeyboardState {
    /// The ctor tail (`:429-433`): the text trims to the max length
    /// (`SetText` → `Utils.TrimUserString`, `UITextBox.cs:64-68`), and an
    /// empty result seeds `KeyState.Shift`.
    pub fn new(request: &Request) -> Self {
        let text: String = request.initial.chars().take(request.max_length).collect();
        let seeded = text.is_empty();
        Self {
            text,
            key_state: if seeded { KeyState::Shift } else { KeyState::Default },
            shift_latched: seeded,
            max_length: request.max_length,
        }
    }

    /// `UITextBox.Backspace` (`UITextBox.cs:76-81`) through the backspace
    /// key's handler (`:392-397`): pop the last character; empty is a no-op.
    pub fn backspace(&mut self) {
        self.text.pop();
    }

    /// `CycleSymbols` (`:933-950`): Default -> Shift -> Symbol -> Default.
    /// The static method is the gamepad path (`PlayerInput`'s cycle, §J.3) —
    /// the App's shift/symbol *clicks* go through [`KeyboardState::
    /// set_key_state`] with the buttons' own `(state != X) ? X : Default`
    /// rule (`:250-254`, `:284-289`), so only the tests consume the cycle.
    #[allow(dead_code)] // The plan Task 6 interface; the click sites route
    // through `set_key_state` (see above).
    pub fn toggle_key_state(&mut self) {
        let next = match self.key_state {
            KeyState::Default => KeyState::Shift,
            KeyState::Shift => KeyState::Symbol,
            KeyState::Symbol => KeyState::Default,
        };
        self.set_key_state(next);
    }

    /// `SetKeyState`'s tail (`:578-582`): the state flips and the latched
    /// button's colour follows (`:581`).
    pub fn set_key_state(&mut self, next: KeyState) {
        self.key_state = next;
        self.shift_latched = next == KeyState::Shift;
    }
}

/// `TypeText` (`:517-533`) — the on-screen letter keys' path: write the
/// character, then unlatch a seeded Shift once the first character lands
/// (`flag && Text.Length > 0 && _keyState == Shift`, `:529-532`).
pub fn type_char(state: &mut KeyboardState, c: char) {
    let was_empty = state.text.is_empty();
    write_str(state, c.to_string().as_str());
    if was_empty && !state.text.is_empty() && state.key_state == KeyState::Shift {
        state.set_key_state(KeyState::Default);
    }
}

/// The no-unlatch write: `_textBox.Write` (`UITextBox.cs:57-61`) as the
/// space bar's `PressSpace` calls it (`:491-502`) and as the physical
/// keyboard's `this.Text = text` lands (`:778-788`) — neither path touches
/// `KeyState`. `SetText` re-trims to the max length (`UITextBox.cs:64-68`).
pub fn write_str(state: &mut KeyboardState, s: &str) {
    state.text = state.text.chars().chain(s.chars()).take(state.max_length).collect();
}

// --- The label tables ----------------------------------------------------------

/// The three label tables (`SetKeyState`, `:562-570`) — 40 characters in the
/// source's row-major order: index `row * 10 + col`, matching the grid fill
/// `DEFAULT_KEYS[j * 10 + i]` (`:223`).
pub fn key_labels(state: KeyState) -> [char; 40] {
    let table = match state {
        KeyState::Default => DEFAULT_KEYS,
        KeyState::Shift => SHIFT_KEYS,
        KeyState::Symbol => SYMBOL_KEYS,
    };
    std::array::from_fn(|i| table.chars().nth(i).unwrap_or('\0'))
}

// --- Geometry -------------------------------------------------------------------

/// `CreateKeyboardButton` (`:655-673`): key `(x, y, width)` -> outer rect.
///
/// `Width.Pixels = 48*w + 4*(w-1)`, `Height.Pixels = 37`, `Left (52*x - 258,
/// 0.5)`, `Top.Pixels = 41*y` — resolved against the panel's **inner** rect
/// (`UIPanel.cs:32`'s 12 px padding), whose `0.5 * 516 = 258` cancels the
/// `-258` exactly. So the columns step 52 px and the rows 41 px from
/// `grid_origin` — the (0, 0) key's top-left — and the grid fills the panel
/// inner flush: the last key's right edge is `542 + 516` and the bottom row's
/// bottom edge `282 + 201`.
pub fn key_rect(grid_origin: [f32; 2], x: usize, y: usize, width: usize) -> [f32; 4] {
    let w = KEY_WIDTH * width as f32 + KEY_SPACING * (width as f32 - 1.0);
    [
        grid_origin[0] + (KEY_WIDTH + KEY_SPACING) * x as f32,
        grid_origin[1] + (KEY_HEIGHT + KEY_SPACING) * y as f32,
        w,
        KEY_HEIGHT,
    ]
}

/// Every static rect of the visible screen, resolved through `uistate::resolve`
/// exactly as the source's construction order does (`:180-425`).
///
/// `submit`/`cancel` are the pair the mode makes visible: the 37 px-tall 0.4
/// pair at the layer's bottom on the gamepad layer (`:291-326`), the 60 px-tall
/// 0.72 pair of `outerLayer2` on the desktop (`:327-390`). `panel` and
/// `grid_origin` describe `outerLayer1` — static geometry the draw only
/// replays when `show_gamepad_hints` holds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Layout {
    /// `outerLayer1`/`outerLayer2` (`:181-194`): `Width 516+8+16 = 540`,
    /// `Top 270`, `Height 266`, `HAlign 0.5`.
    pub layer: [f32; 4],
    /// The key panel (`:196-207`): `Width 1.0` of the layer, `Height 225`,
    /// bg `(23, 33, 69) * 0.7`.
    pub panel: [f32; 4],
    /// The textbox (`:210-218`): `Width 516`, `Top -50 + 270 - 10 = 210`
    /// (`+50` without hints, `:716-725`), `Height 37` clamped up to
    /// `SetText`'s `MinHeight = 32*0.78 + 12 + 12 = 48.96`
    /// (`UITextPanel.cs:133-134` over `UIPanel.cs:32`'s padding).
    pub textbox: [f32; 4],
    /// The label (`:408-425`): `Width 516`, `Top -50 - 37 - 4 + 270 = 179`
    /// (`+50`), `Height 37` (`MinHeight 32*0.75 = 24` loses).
    pub label: [f32; 4],
    /// The visible Submit rect (mode-dependent, see the struct doc).
    pub submit: [f32; 4],
    /// The visible Cancel rect.
    pub cancel: [f32; 4],
    /// The (0, 0) key's top-left — `(542, 282)`; only meaningful with hints.
    pub grid_origin: [f32; 2],
}

/// The whole layout at `screen` (the state root is the full window — the
/// `UIState`'s `GetInnerDimensions`). `show_gamepad_hints` picks which layer
/// `DrawSelf` keeps alive (`:684-726`): with it, `outerLayer1` and the
/// textbox/label sit at their unshifted pixel tops; without it, `outerLayer2`
/// and the `+50` shift (`:716-725`).
pub fn layout(screen: Vec2, show_gamepad_hints: bool) -> Layout {
    let root = [0.0, 0.0, screen.x, screen.y];
    // `outerLayer1` == `outerLayer2` — the identical initializer (`:180-195`).
    let layer = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(270.0),      // `num2 = 270f` -> `Top.Pixels` (`:177`, `:183`)
        Style::px(540.0),      // `num4 + 8f + 16f` = 516 + 24 (`:179`, `:181`)
        Style::px(266.0),      // `:184`
        0.5,                   // `uIElement.HAlign = 0.5f` (`:185`)
        0.0,
        0.0,
        0.0,
        root[2],
        [0.0; 4],              // `uIElement.SetPadding(0f)` (`:186`)
    );
    // The key panel: `Width.Precent = 1`, `Height.Pixels = 225` (`:198-205`),
    // a child of `outerLayer1` with its own `UIPanel` padding 12.
    let layer_inner = uistate::inner(layer, [0.0; 4]);
    let panel = uistate::resolve(
        layer_inner,
        Style::px(0.0),
        Style::px(0.0),
        Style::pct(1.0),
        Style::px(225.0),
        0.0,
        0.0,
        0.0,
        0.0,
        layer_inner[2],
        [0.0; 4],
    );
    let panel_inner = uistate::inner(panel, UI_PANEL_PADDING);
    // The (0, 0) key — `Left (52*0 - 258, 0.5)`, `Top 41*0` (`:661-663`).
    let key00 = uistate::resolve(
        panel_inner,
        Style::new(-258.0, 0.5), // `Left.Pixels = 52*x - num*0.5`, `Left.Precent = 0.5`
        Style::px(0.0),
        Style::px(KEY_WIDTH),
        Style::px(KEY_HEIGHT),
        0.0,
        0.0,
        0.0,
        0.0,
        panel_inner[2],
        [0.0; 4],
    );
    let grid_origin = [key00[0], key00[1]];
    // The DrawSelf shift (`:716-725`): `+50` px on both tops when the desktop
    // layer is the live one.
    let shift = if show_gamepad_hints { 0.0 } else { 50.0 };
    // The textbox (`:210-218`): `Top.Pixels = num5 + num2 - 10 + num3` with
    // `num5 = -50`, `num2 = 270`, `num3 = 0` (the sign offset is context 1
    // only, `:176`) -> 210.
    let textbox = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(210.0 + shift),
        Style::px(516.0), // `num4` (`:214`)
        Style::px(37.0),  // `:217`
        0.5,              // `this._textBox.HAlign = 0.5f` (`:213`)
        0.0,
        24.0,                        // MinWidth of the empty text + padding (`UITextPanel.cs:133`)
        32.0 * TEXTBOX_SCALE + 24.0, // MinHeight (`UITextPanel.cs:134`)
        root[2],
        [0.0; 4],
    );
    // The label (`:408-425`): `Top.Pixels = num5 - 37 - 4 + num2 + num3` -> 179.
    let label = uistate::resolve(
        root,
        Style::px(0.0),
        Style::px(179.0 + shift),
        Style::px(516.0), // `Width.Pixels = num4` (`:413`)
        Style::px(37.0),  // `:422`
        0.5,              // `HAlign = 0.5f` (`:410`)
        0.0,
        0.0,                // MinWidth growth documented off (module header)
        32.0 * LABEL_SCALE, // MinHeight, padding 0 (`UIText` is a bare `UIElement`)
        root[2],
        [0.0; 4],
    );
    // The mode's Submit/Cancel pair (see the struct doc).
    let (submit, cancel) = if show_gamepad_hints {
        // `_submitButton`/`_cancelButton` (`:291-326`): `Width 0.4`, `Height
        // 37`, `VAlign 1`, `HAlign 1` / default 0 — the layer's bottom corners.
        (
            uistate::resolve(
                layer_inner,
                Style::px(0.0),
                Style::px(0.0),
                Style::pct(0.4),
                Style::px(37.0),
                1.0,
                1.0,
                0.0,
                0.0,
                layer_inner[2],
                [0.0; 4],
            ),
            uistate::resolve(
                layer_inner,
                Style::px(0.0),
                Style::px(0.0),
                Style::pct(0.4),
                Style::px(37.0),
                0.0,
                1.0,
                0.0,
                0.0,
                layer_inner[2],
                [0.0; 4],
            ),
        )
    } else {
        // `_submitButton2`/`_cancelButton2` (`:327-390`): `Width 0.4`,
        // `Height 60`, `Top 50` / `114`, `HAlign 0.5`, `VAlign 0`.
        (
            uistate::resolve(
                layer_inner,
                Style::px(0.0),
                Style::px(50.0),
                Style::pct(0.4),
                Style::px(60.0),
                0.5,
                0.0,
                0.0,
                0.0,
                layer_inner[2],
                [0.0; 4],
            ),
            uistate::resolve(
                layer_inner,
                Style::px(0.0),
                Style::px(114.0),
                Style::pct(0.4),
                Style::px(60.0),
                0.5,
                0.0,
                0.0,
                0.0,
                layer_inner[2],
                [0.0; 4],
            ),
        )
    };
    Layout {
        layer,
        panel,
        textbox,
        label,
        submit,
        cancel,
        grid_origin,
    }
}

// --- Colours ---------------------------------------------------------------------

/// Key styling (`StyleKey`, `:628-652` + `SetKeyState`'s latch `:579-582`):
/// rest bg `(63, 82, 151) * 0.7` and border `_internalBorderColor * 0.7`;
/// hover bg `(73, 94, 171) * 0.85` and border
/// `_internalBorderColorSelected * 0.85` (`_internalBorderColorSelected` is
/// `Main.OurFavoriteColor`, `:70`); a latched button's bg is `(93, 114, 191)`
/// (`:581`) and hover does not repaint it (`:237-240`'s `keyState != Shift`
/// gate). Returns `(bg, border)` as pre-tint bytes.
pub fn key_colors(latched: bool, hovered: bool) -> ([u8; 4], [u8; 4]) {
    let bg = if latched {
        [93, 114, 191, 255] // `:581`
    } else if hovered {
        color::mul([73, 94, 171, 255], 0.85) // `:639`
    } else {
        color::mul([63, 82, 151, 255], 0.7) // `:632`
    };
    let border = if hovered {
        color::mul(uistate::OUR_FAVORITE_COLOR, 0.85) // `:642`
    } else {
        color::mul(INTERNAL_BORDER_COLOR, 0.7) // `:635`
    };
    (bg, border)
}

/// The shift/symbol buttons' own hover pair (`:234-249`, `:268-283`) — the
/// full `(73, 94, 171)` hover bg (no `* 0.85`) and the full
/// `_internalBorderColorSelected` border, unlike [`key_colors`]' `StyleKey`
/// maths. A latched toggle still paints `(93, 114, 191)` (`:581`).
fn toggle_colors(latched: bool, hovered: bool) -> ([u8; 4], [u8; 4]) {
    let bg = if latched {
        [93, 114, 191, 255] // `:581`
    } else if hovered {
        [73, 94, 171, 255] // `:239`, `:273`
    } else {
        color::mul([63, 82, 151, 255], 0.7) // `:232`, `:266`
    };
    let border = if hovered {
        uistate::OUR_FAVORITE_COLOR // `:236`, `:270`
    } else {
        color::mul(INTERNAL_BORDER_COLOR, 0.7) // `:233`, `:267`
    };
    (bg, border)
}

/// `ValidateText` (`:585-613`) + `FadedMouseOver`/`FadedMouseOut`
/// (`:1069-1080`): a valid submit draws White text over the normal pair —
/// hover bg `(73, 94, 171)` (`:593`), idle `(63, 82, 151) * 0.7` (`:597`);
/// an invalid one draws Gray over `(180, 60, 60) * 0.85` (hover, `:606`) /
/// `(150, 40, 40) * 0.85` (idle, `:610`). The border is
/// `FancyUIFatButtonMouseOver` while hovered (both FadedMouseOver calls run
/// before `ValidateText`'s bg write, `:299-308`) and Black otherwise.
/// Returns `(bg, border)`; the text colour is White iff `valid` (`:590`,
/// `:603`).
pub fn submit_colors(valid: bool, hovered: bool) -> ([u8; 4], [u8; 4]) {
    let bg = match (valid, hovered) {
        (true, true) => [73, 94, 171, 255],                     // `:593`
        (true, false) => color::mul([63, 82, 151, 255], 0.7),   // `:597`
        (false, true) => color::mul([180, 60, 60, 255], 0.85),  // `:606`
        (false, false) => color::mul([150, 40, 40, 255], 0.85), // `:610`
    };
    let border = if hovered {
        uistate::FANCY_UI_FAT_BUTTON_MOUSE_OVER // `:1073`
    } else {
        [0, 0, 0, 255] // `:1079`
    };
    (bg, border)
}

/// `TextIsValidForSubmit` (`:615-626`) with the custom validator unset and
/// the sign/chest contexts out of the port (the App's keyboards are all
/// context 0): a trimmed-empty string submits iff `allowEmpty`.
pub fn text_valid(text: &str, allow_empty: bool) -> bool {
    !text.trim().is_empty() || allow_empty
}

// --- Clicks -----------------------------------------------------------------------

/// What a click on the visible layer does (`:291-326`, `:327-390`, the grid
/// `:219-227`, the bottom row `:228`, `:263`, `:452`, `:391`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Click {
    /// A letter/symbol grid key — the character its current label shows.
    Char(char),
    /// The shift button (`:250-254`).
    Shift,
    /// The symbol button (`:284-289`).
    Symbol,
    /// The space bar (`:452-457`).
    Space,
    /// The backspace key (`:391-397`).
    Backspace,
    /// Submit — reported even when the text is invalid; the *handler* re-checks
    /// [`text_valid`] (`:358-365`, `:970-978`).
    Submit,
    /// Cancel (`:319-323`, `:385-389`).
    Cancel,
    /// Nothing.
    None,
}

fn in_rect(mouse: Vec2, rect: [f32; 4]) -> bool {
    mouse.x >= rect[0]
        && mouse.x < rect[0] + rect[2]
        && mouse.y >= rect[1]
        && mouse.y < rect[1] + rect[3]
}

/// The layer's hit-test — the same [`key_rect`] geometry the draw replays
/// (Review Focus 2). The visible layer's buttons only: `outerLayer1`'s grid
/// and bottom row plus the small Submit/Cancel pair while
/// `show_gamepad_hints` holds (`:684-704`), just the big pair of
/// `outerLayer2` otherwise (`:705-726`).
pub fn click_at(state: &KeyboardState, l: &Layout, show_gamepad_hints: bool, mouse: Vec2) -> Click {
    if show_gamepad_hints {
        // The bottom row first (`:228`, `:263`, `:452`, `:391`).
        if in_rect(mouse, key_rect(l.grid_origin, 0, 4, 1)) {
            return Click::Shift;
        }
        if in_rect(mouse, key_rect(l.grid_origin, 1, 4, 1)) {
            return Click::Symbol;
        }
        if in_rect(mouse, key_rect(l.grid_origin, 2, 4, 6)) {
            return Click::Space;
        }
        if in_rect(mouse, key_rect(l.grid_origin, 8, 4, 2)) {
            return Click::Backspace;
        }
        // The 40 letter keys (`:219-227`) — labels follow the live `KeyState`.
        let labels = key_labels(state.key_state);
        for row in 0..4 {
            for col in 0..10 {
                if in_rect(mouse, key_rect(l.grid_origin, col, row, 1)) {
                    return Click::Char(labels[row * 10 + col]);
                }
            }
        }
        // The small pair at the layer's bottom (`:313`, `:326`).
        if in_rect(mouse, l.submit) {
            return Click::Submit;
        }
        if in_rect(mouse, l.cancel) {
            return Click::Cancel;
        }
        return Click::None;
    }
    // The desktop layer: only `outerLayer2`'s pair (`:366`, `:390`).
    if in_rect(mouse, l.submit) {
        return Click::Submit;
    }
    if in_rect(mouse, l.cancel) {
        return Click::Cancel;
    }
    Click::None
}

// --- Draw ---------------------------------------------------------------------------

/// The Death-font width a large `UITextPanel` measures (`UITextPanel.cs:126-131`),
/// or the bitmap fallback's.
fn large_text_width(r: &SpriteRenderer, lookups: &assets::Lookups, text: &str) -> f32 {
    match &lookups.death_font {
        Some(f) => font_width(f, text, 1.0),
        None => r.text_width(text, 1.0),
    }
}

/// `Utils.DrawBorderStringBig` (`Utils.cs:2520-2535`) through the Death font:
/// the 3x3 black ring, then the colour on top.
fn border_string_big(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    x: f32,
    y: f32,
    scale: f32,
    color: [f32; 4],
    text: &str,
) {
    for dy in -1i32..=1 {
        for dx in -1i32..=1 {
            match &lookups.death_font {
                Some(f) => push_font_text(
                    r,
                    f,
                    x + dx as f32,
                    y + dy as f32,
                    scale,
                    [0.0, 0.0, 0.0, 1.0],
                    text,
                ),
                None => r.push_text(x + dx as f32, y + dy as f32, scale, [0.0, 0.0, 0.0, 1.0], text),
            }
        }
    }
    match &lookups.death_font {
        Some(f) => push_font_text(r, f, x, y, scale, color, text),
        None => r.push_text(x, y, scale, color, text),
    }
}

/// `UITextPanel.DrawSelf` (`UITextPanel.cs:139-146`) + `DrawText` (`:161-169`)
/// for the keyboard's panels — bg 9-slice, border 9-slice, then the 0.4-scale
/// Death-font label at `TextDrawPosition` (`:92-108`): centred against the
/// inner width (`TextHAlign = 0.5`, `:22`) with the `10*scale^2` large-font
/// lift, over the caller's `(left, top, right, bottom)` padding (the keyboard
/// zeroes only the sides `StyleKey`/the ctors zero).
#[allow(clippy::too_many_arguments)]
fn draw_panel_text(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    pad: [f32; 4],
    bg: [u8; 4],
    border: [u8; 4],
    scale: f32,
    text: &str,
    text_color: [f32; 4],
) {
    panel::draw_panel(r, lookups, lookups.ui_panel_bg, rect, color::tint(bg));
    panel::draw_panel(r, lookups, lookups.ui_panel_border, rect, color::tint(border));
    draw_panel_label(r, lookups, rect, pad, scale, text, text_color);
}

/// The `DrawText` half of [`draw_panel_text`] — the text without the panel
/// (`UITextPanel.DrawPanel == false`, `:329`, `:369`).
fn draw_panel_label(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    rect: [f32; 4],
    pad: [f32; 4],
    scale: f32,
    text: &str,
    text_color: [f32; 4],
) {
    let inner = uistate::inner(rect, pad);
    let text_w = large_text_width(r, lookups, text);
    // `TextDrawPosition` (`UITextPanel.cs:92-108`).
    let tx = inner[0] + (inner[2] - text_w * scale) * 0.5;
    let ty = inner[1] - 10.0 * scale * scale;
    border_string_big(r, lookups, tx, ty, scale, text_color, text);
}

/// `UIImage`'s centred draw: the sheet at `ImageScale`, centred on the host
/// rect's centre (`UIImage.DrawSelf` — `position + size/2` origin,
/// `ImageScale` scale). The shift icon is `0.85` (`:255-261`), the backspace
/// icon `0.92` (`:401-406`). `None` sheet draws nothing (Review Focus 7).
fn draw_icon(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    sheet: Option<SheetRect>,
    rect: [f32; 4],
    image_scale: f32,
) {
    let Some(sheet) = sheet else { return };
    let (aw, ah) = (lookups.atlas_size.0 as f32, lookups.atlas_size.1 as f32);
    let w = sheet.width as f32 * image_scale;
    let h = sheet.height as f32 * image_scale;
    let cx = rect[0] + rect[2] * 0.5;
    let cy = rect[1] + rect[3] * 0.5;
    let uv = [
        sheet.x as f32 / aw,
        sheet.y as f32 / ah,
        (sheet.x as f32 + sheet.width as f32) / aw,
        (sheet.y as f32 + sheet.height as f32) / ah,
    ];
    r.push_ui_sprite([cx - w * 0.5, cy - h * 0.5], [w, h], uv, [1.0; 4]);
}

/// The whole screen in the source's draw order. The `base` children append in
/// ctor order — the textbox (`:218`), the label (`:425`), then the live layer
/// (`:427`/`:710`) — and `DrawSelf` swaps the layers on
/// `ShowGamepadHints` (`:684-726`). `frame` drives the textbox caret's blink
/// (`UITextBox._frameCount % 40 <= 20`, `UITextBox.cs:94-108`); the App passes
/// its frame counter.
///
/// The textbox text colour is White: `DrawSelf` lerps toward the
/// `Main.tileColor` dim (`:798-801`), which is White in the menu (no tile
/// under the cursor — `(255 + 255*2)/3 = 255`).
pub fn draw(
    r: &mut SpriteRenderer,
    lookups: &assets::Lookups,
    state: &KeyboardState,
    request: &Request,
    l: &Layout,
    show_gamepad_hints: bool,
    mouse: Vec2,
    frame: u64,
) {
    // The textbox (`UITextBox.DrawSelf`, `UITextBox.cs:80-108`): a transparent
    // panel (`:211-212`) so only the text shows, centred, with the caret.
    draw_panel_label(
        r,
        lookups,
        l.textbox,
        UI_PANEL_PADDING,
        TEXTBOX_SCALE,
        &state.text,
        [1.0, 1.0, 1.0, 1.0],
    );
    // The caret (`UITextBox.cs:96-108`): `_cursor` is always `Text.Length`
    // (`:82`), the blink is `frame % 40 <= 20`, the offset `+6 - 8*scale` x
    // and `+2*scale` y from the text position.
    if frame % 40 <= 20 {
        let inner = uistate::inner(l.textbox, UI_PANEL_PADDING);
        let text_w = large_text_width(r, lookups, &state.text);
        let tx = inner[0] + (inner[2] - text_w * TEXTBOX_SCALE) * 0.5;
        let ty = inner[1] - 10.0 * TEXTBOX_SCALE * TEXTBOX_SCALE;
        border_string_big(
            r,
            lookups,
            tx + text_w * TEXTBOX_SCALE + (6.0 - 8.0 * TEXTBOX_SCALE),
            ty + 2.0 * TEXTBOX_SCALE,
            TEXTBOX_SCALE,
            [1.0, 1.0, 1.0, 1.0],
            "|",
        );
    }
    // The label (`UIText.DrawSelf`, `UIText.cs:137-166`): `TextOriginX 0.5`
    // centres it, the large font lifts `10*scale`, the ChatManager shadow is
    // 4 directional passes at spread 1.5 (`ChatManager.cs:200-205`, `:376-383`;
    // `UIText.cs:214`, `_shadowColor` Black `:25`).
    let label_w = large_text_width(r, lookups, &request.label);
    let lx = l.label[0] + (l.label[2] - label_w * LABEL_SCALE) * 0.5;
    let ly = l.label[1] - 10.0 * LABEL_SCALE;
    for (dx, dy) in [(-1.5f32, 0.0), (1.5, 0.0), (0.0, -1.5), (0.0, 1.5)] {
        match &lookups.death_font {
            Some(f) => push_font_text(
                r,
                f,
                lx + dx,
                ly + dy,
                LABEL_SCALE,
                [0.0, 0.0, 0.0, 1.0],
                &request.label,
            ),
            None => r.push_text(lx + dx, ly + dy, LABEL_SCALE, [0.0, 0.0, 0.0, 1.0], &request.label),
        }
    }
    match &lookups.death_font {
        Some(f) => push_font_text(r, f, lx, ly, LABEL_SCALE, [1.0, 1.0, 1.0, 1.0], &request.label),
        None => r.push_text(lx, ly, LABEL_SCALE, [1.0, 1.0, 1.0, 1.0], &request.label),
    }

    if show_gamepad_hints {
        // `outerLayer1` (`:684-704`): the panel, its keys in append order —
        // the 40 letters (`:219-227`), shift (`:262`), symbol (`:289`), space
        // (`:290`), backspace (`:407`) — then the small Submit/Cancel pair
        // (`:313`, `:326`).
        panel::draw_panel(r, lookups, lookups.ui_panel_bg, l.panel, color::tint(PANEL_BG));
        panel::draw_panel(
            r,
            lookups,
            lookups.ui_panel_border,
            l.panel,
            color::tint(uistate::TEXT_PANEL_BORDER),
        );
        // The letter keys: `StyleKey`'d (`:664-666` -> `:630-631` zeroes the
        // side padding; top/bottom keep the 12 px default).
        let labels = key_labels(state.key_state);
        for row in 0..4 {
            for col in 0..10 {
                let rect = key_rect(l.grid_origin, col, row, 1);
                let (bg, border) = key_colors(false, in_rect(mouse, rect));
                draw_panel_text(
                    r,
                    lookups,
                    rect,
                    [0.0, 12.0, 0.0, 12.0],
                    bg,
                    border,
                    KEY_TEXT_SCALE,
                    &labels[row * 10 + col].to_string(),
                    [1.0; 4],
                );
            }
        }
        // The shift button (`:228-262`): no text, the `VK_Shift` icon at
        // `ImageScale 0.85`, latched while `KeyState::Shift`.
        let shift_rect = key_rect(l.grid_origin, 0, 4, 1);
        let (bg, border) = toggle_colors(state.key_state == KeyState::Shift, in_rect(mouse, shift_rect));
        draw_panel_text(
            r,
            lookups,
            shift_rect,
            [0.0; 4],
            bg,
            border,
            KEY_TEXT_SCALE,
            "",
            [1.0; 4],
        );
        draw_icon(r, lookups, lookups.vk_shift, shift_rect, 0.85);
        // The symbol button (`:263-289`): the "@%" label, latched while
        // `KeyState::Symbol`.
        let symbol_rect = key_rect(l.grid_origin, 1, 4, 1);
        let (bg, border) =
            toggle_colors(state.key_state == KeyState::Symbol, in_rect(mouse, symbol_rect));
        draw_panel_text(
            r,
            lookups,
            symbol_rect,
            [0.0, 12.0, 0.0, 12.0],
            bg,
            border,
            KEY_TEXT_SCALE,
            SYMBOL_BUTTON_LABEL,
            [1.0; 4],
        );
        // The space bar (`:452-457`): the `UI.SpaceButton` label, 6 keys wide.
        let space_rect = key_rect(l.grid_origin, 2, 4, 6);
        let (bg, border) = key_colors(false, in_rect(mouse, space_rect));
        draw_panel_text(
            r,
            lookups,
            space_rect,
            [0.0, 12.0, 0.0, 12.0],
            bg,
            border,
            KEY_TEXT_SCALE,
            SPACE,
            [1.0; 4],
        );
        // The backspace key (`:391-407`): the `VK_Backspace` icon at
        // `ImageScale 0.92`, 2 keys wide.
        let backspace_rect = key_rect(l.grid_origin, 8, 4, 2);
        let (bg, border) = key_colors(false, in_rect(mouse, backspace_rect));
        draw_panel_text(
            r,
            lookups,
            backspace_rect,
            [0.0; 4],
            bg,
            border,
            KEY_TEXT_SCALE,
            "",
            [1.0; 4],
        );
        draw_icon(r, lookups, lookups.vk_backspace, backspace_rect, 0.92);
        // The small pair: Submit carries `ValidateText`'s colours (`:298`,
        // `:585-613`), Cancel is `StyleKey(external: true)`'d — the border
        // stays the `UIPanel` default Black (`:315`, `:634-636`'s `!external`).
        let valid = text_valid(&state.text, request.allow_empty);
        let (bg, border) = submit_colors(valid, in_rect(mouse, l.submit));
        draw_panel_text(
            r,
            lookups,
            l.submit,
            [0.0, 12.0, 0.0, 12.0],
            bg,
            border,
            KEY_TEXT_SCALE,
            SUBMIT,
            if valid { [1.0; 4] } else { color::tint(GRAY) },
        );
        let (bg, border) = if in_rect(mouse, l.cancel) {
            ([73, 94, 171, 255], uistate::FANCY_UI_FAT_BUTTON_MOUSE_OVER) // `:1072-1073`
        } else {
            (color::mul([63, 82, 151, 255], 0.7), [0, 0, 0, 255]) // `:1078-1079`
        };
        draw_panel_text(
            r,
            lookups,
            l.cancel,
            [0.0, 12.0, 0.0, 12.0],
            bg,
            border,
            KEY_TEXT_SCALE,
            CANCEL,
            [1.0; 4],
        );
    } else {
        // `outerLayer2` (`:705-726`): the 0.72 pair only — `DrawPanel` is
        // false (`:329`, `:369`), so they are text: Silver at 0.72, White at
        // 0.85 while hovered (`:334-343`, `:370-379`).
        let submit_hover = in_rect(mouse, l.submit);
        draw_panel_label(
            r,
            lookups,
            l.submit,
            [0.0, 12.0, 0.0, 12.0],
            if submit_hover { SUBMIT2_HOVER_SCALE } else { SUBMIT2_SCALE },
            SUBMIT,
            if submit_hover { [1.0; 4] } else { color::tint(SILVER) },
        );
        let cancel_hover = in_rect(mouse, l.cancel);
        draw_panel_label(
            r,
            lookups,
            l.cancel,
            [0.0, 12.0, 0.0, 12.0],
            if cancel_hover { SUBMIT2_HOVER_SCALE } else { SUBMIT2_SCALE },
            CANCEL,
            if cancel_hover { [1.0; 4] } else { color::tint(SILVER) },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Expected values hand-derived from the C# (`UIVirtualKeyboard.cs:159-746`,
    // `UITextPanel.cs:92-134`, `UIPanel.cs:18-32`, `Utils.cs:2520-2535`,
    // `UITextBox.cs:57-108`, `ChatManager.cs:200-205,376-383`) — never from
    // this code.

    const SCREEN: Vec2 = Vec2::new(1600.0, 900.0);

    #[test]
    fn layout_matches_the_source_table_at_1600x900() {
        // The desktop layer (`show_gamepad_hints == false`) — the screen the
        // port always shows (Review Focus 6).
        let l = layout(SCREEN, false);
        // outerLayer: Width 516+8+16 = 540, Top 270, Height 266, HAlign .5
        // (`:181-185`) -> x = 800 - 270 = 530.
        assert_eq!(l.layer, [530.0, 270.0, 540.0, 266.0]);
        // The panel: Width 1.0 of the zero-padded layer, Height 225 (`:196-207`).
        assert_eq!(l.panel, [530.0, 270.0, 540.0, 225.0]);
        // The textbox: Top -50+270-10 = 210, +50 on the desktop layer
        // (`:716-725`) -> 260; the Height 37 clamps up to SetText's
        // MinHeight = 32*0.78 + 12 + 12 = 48.96 (`UITextPanel.cs:133-134`).
        assert_eq!(l.textbox, [542.0, 260.0, 516.0, 48.96]);
        // The label: Top -50-37-4+270 = 179, +50 -> 229; MinHeight 24 loses
        // to the 37 (`:408-425`).
        assert_eq!(l.label, [542.0, 229.0, 516.0, 37.0]);
        // The 0.72 pair of outerLayer2: W 0.4*540 = 216, Top 50/114, HAlign .5
        // (`:327-344`, `:367-390`).
        assert_eq!(l.submit, [692.0, 320.0, 216.0, 60.0]);
        assert_eq!(l.cancel, [692.0, 384.0, 216.0, 60.0]);
        // The grid origin: the (0,0) key over the panel inner (padding 12) —
        // `Left (-258, 0.5)` cancels against `0.5*516` -> panel x + 12 = 542;
        // `Top 41*0` -> panel y + 12 = 282 (`:661-663`, `UIPanel.cs:32`).
        assert_eq!(l.grid_origin, [542.0, 282.0]);

        // The gamepad layer (`show_gamepad_hints == true`): the tops lose the
        // +50 and the small 37-tall pair sits at the layer's bottom
        // (`:291-326`).
        let g = layout(SCREEN, true);
        assert_eq!(g.textbox, [542.0, 210.0, 516.0, 48.96]);
        assert_eq!(g.label, [542.0, 179.0, 516.0, 37.0]);
        // Submit: W 216, H 37, HAlign 1 + VAlign 1 of the 540x266 layer
        // -> (530 + 540 - 216, 270 + 266 - 37).
        assert_eq!(g.submit, [854.0, 499.0, 216.0, 37.0]);
        assert_eq!(g.cancel, [530.0, 499.0, 216.0, 37.0]);
        // The grid geometry is mode-independent.
        assert_eq!(g.grid_origin, l.grid_origin);
        assert_eq!(g.panel, l.panel);
    }

    #[test]
    fn key_rect_matches_the_create_keyboard_button_oracle() {
        let origin = [542.0, 282.0];
        // `Width 48*w + 4*(w-1)`, `Height 37`, `Left 52*x - 258 (+0.5*516)`,
        // `Top 41*y` (`:655-673`).
        assert_eq!(key_rect(origin, 0, 0, 1), [542.0, 282.0, 48.0, 37.0]);
        assert_eq!(key_rect(origin, 9, 3, 1), [1010.0, 405.0, 48.0, 37.0]);
        // The grid fills the panel inner flush: the last key's right edge is
        // 542 + 516, the bottom row's bottom edge 282 + 201.
        assert_eq!(key_rect(origin, 9, 0, 1)[0] + 48.0, 542.0 + 516.0);
        assert_eq!(key_rect(origin, 0, 4, 1)[1] + 37.0, 282.0 + 201.0);
        // The bottom row: shift (0,4), symbol (1,4), space (2,4,6), backspace
        // (8,4,2) (`:228`, `:263`, `:452`, `:391`).
        assert_eq!(key_rect(origin, 0, 4, 1), [542.0, 446.0, 48.0, 37.0]);
        assert_eq!(key_rect(origin, 1, 4, 1), [594.0, 446.0, 48.0, 37.0]);
        assert_eq!(key_rect(origin, 2, 4, 6), [646.0, 446.0, 308.0, 37.0]);
        assert_eq!(key_rect(origin, 8, 4, 2), [958.0, 446.0, 100.0, 37.0]);
    }

    #[test]
    fn key_labels_match_the_set_key_state_tables() {
        // `SetKeyState`'s three strings (`:562-570`), read row-major like the
        // grid fill `DEFAULT_KEYS[j * 10 + i]` (`:223`).
        let d = key_labels(KeyState::Default);
        assert_eq!(&d[0..10], &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
        assert_eq!(d[10], 'q');
        assert_eq!(d[19], 'p');
        assert_eq!(d[20], 'a');
        assert_eq!(d[29], '\'');
        assert_eq!(d[30], 'z');
        assert_eq!(d[39], '?');
        // Shift is the uppercase twin (`:34`).
        let s = key_labels(KeyState::Shift);
        assert_eq!(&s[0..10], &['1', '2', '3', '4', '5', '6', '7', '8', '9', '0']);
        assert_eq!(s[10], 'Q');
        assert_eq!(s[39], '?');
        // Symbol band 1 is `!@#$%^&*()` (`:36`).
        let y = key_labels(KeyState::Symbol);
        assert_eq!(&y[10..20], &['!', '@', '#', '$', '%', '^', '&', '*', '(', ')']);
        assert_eq!(&y[20..30], &['-', '_', '+', '=', '/', '\\', '{', '}', '[', ']']);
        assert_eq!(y[39], '¥');
    }

    #[test]
    fn key_colors_recompute_through_the_xna_byte_maths() {
        // Rest: `(63,82,151)*0.7` / `(89,116,213)*0.7` (`:632`, `:635`).
        assert_eq!(
            key_colors(false, false),
            (color::mul([63, 82, 151, 255], 0.7), color::mul([89, 116, 213, 255], 0.7))
        );
        assert_eq!(key_colors(false, false).0, [44, 57, 105, 178]);
        assert_eq!(key_colors(false, false).1, [62, 81, 149, 178]);
        // Hover: `(73,94,171)*0.85` / `OurFavoriteColor*0.85` (`:639`, `:642`).
        assert_eq!(
            key_colors(false, true),
            (color::mul([73, 94, 171, 255], 0.85), color::mul([255, 231, 69, 255], 0.85))
        );
        assert_eq!(key_colors(false, true).0, [62, 79, 145, 216]);
        assert_eq!(key_colors(false, true).1, [216, 196, 58, 216]);
        // Latched: `(93,114,191)` wins over the hover repaint (`:581`,
        // `:237-240`); the border still follows the hover (`:642`).
        assert_eq!(key_colors(true, false).0, [93, 114, 191, 255]);
        assert_eq!(key_colors(true, true).0, [93, 114, 191, 255]);
        // The toggle buttons' own pair: the full `(73,94,171)` hover and the
        // full `_internalBorderColorSelected` border (`:239`, `:236`).
        assert_eq!(toggle_colors(false, true), ([73, 94, 171, 255], [255, 231, 69, 255]));
        assert_eq!(toggle_colors(true, true).0, [93, 114, 191, 255]);
        assert_eq!(toggle_colors(false, false).0, [44, 57, 105, 178]);
    }

    #[test]
    fn submit_colors_match_validate_text() {
        // Valid: White text over the normal pair (`:590-598`).
        assert_eq!(submit_colors(true, false).0, [44, 57, 105, 178]);
        assert_eq!(submit_colors(true, false).1, [0, 0, 0, 255]);
        assert_eq!(submit_colors(true, true).0, [73, 94, 171, 255]);
        assert_eq!(submit_colors(true, true).1, [255, 231, 69, 255]);
        // Invalid: Gray text over the red pair (`:603-611`); the border stays
        // the FadedMouseOver pair while hovered (`:299-308`).
        assert_eq!(submit_colors(false, false).0, color::mul([150, 40, 40, 255], 0.85));
        assert_eq!(submit_colors(false, false).0, [127, 34, 34, 216]);
        assert_eq!(submit_colors(false, true).0, color::mul([180, 60, 60, 255], 0.85));
        assert_eq!(submit_colors(false, true).0, [153, 51, 51, 216]);
        assert_eq!(submit_colors(false, true).1, [255, 231, 69, 255]);
    }

    #[test]
    fn text_valid_follows_the_trim_rule() {
        // `TextIsValidForSubmit` (`:615-626`): the trimmed length decides.
        assert!(!text_valid("", false));
        assert!(!text_valid("   ", false));
        assert!(text_valid("a", false));
        assert!(text_valid("", true));
        assert!(text_valid("  ", true));
    }

    #[test]
    fn new_state_seeds_shift_on_empty_text() {
        // `:429-433` — empty starting text seeds `KeyState.Shift`.
        let seeded = KeyboardState::new(&Request {
            label: ENTER_CHARACTER_NAME.into(),
            initial: String::new(),
            allow_empty: true,
            max_length: 20,
        });
        assert_eq!(seeded.key_state, KeyState::Shift);
        assert!(seeded.shift_latched);
        assert_eq!(seeded.text, "");
        // A non-empty start stays Default.
        let named = KeyboardState::new(&Request {
            label: ENTER_WORLD_NAME.into(),
            initial: "World 1".into(),
            allow_empty: true,
            max_length: 27,
        });
        assert_eq!(named.key_state, KeyState::Default);
        assert!(!named.shift_latched);
        assert_eq!(named.text, "World 1");
        // The initial trims to the max length (`UITextBox.cs:64-68`).
        let capped = KeyboardState::new(&Request {
            label: String::new(),
            initial: "0123456789".into(),
            allow_empty: false,
            max_length: 4,
        });
        assert_eq!(capped.text, "0123");
    }

    #[test]
    fn type_char_unlatches_the_seeded_shift_after_the_first_char() {
        // `TypeText` (`:526-532`): the first character lands, then the seeded
        // Shift drops; later characters keep it down.
        let mut state = KeyboardState::new(&Request {
            label: String::new(),
            initial: String::new(),
            allow_empty: true,
            max_length: 20,
        });
        type_char(&mut state, 'h');
        assert_eq!(state.text, "h");
        assert_eq!(state.key_state, KeyState::Default);
        assert!(!state.shift_latched);
        type_char(&mut state, 'i');
        assert_eq!(state.text, "hi");
        assert_eq!(state.key_state, KeyState::Default);
        // `backspace` pops (`:392-397`) and never re-latches.
        state.backspace();
        assert_eq!(state.text, "h");
        assert_eq!(state.key_state, KeyState::Default);
        // Empty is a no-op (`UITextBox.cs:76-81`).
        state.backspace();
        state.backspace();
        assert_eq!(state.text, "");
    }

    #[test]
    fn write_str_never_unlatches_and_caps_the_length() {
        // The `PressSpace`/`Main.GetInputText` path (`:491-502`, `:778-788`)
        // writes without touching `KeyState`.
        let mut state = KeyboardState::new(&Request {
            label: String::new(),
            initial: String::new(),
            allow_empty: true,
            max_length: 8,
        });
        assert_eq!(state.key_state, KeyState::Shift);
        write_str(&mut state, "ab");
        assert_eq!(state.text, "ab");
        assert_eq!(state.key_state, KeyState::Shift, "the physical path keeps Shift");
        // `SetTextMaxLength` caps (`UITextBox.cs:64-68`) — and the space the
        // cap swallows stays swallowed (`:491-502` writes through the same
        // `SetText`).
        write_str(&mut state, "cdefghij");
        assert_eq!(state.text, "abcdefgh");
        write_str(&mut state, " ");
        assert_eq!(state.text, "abcdefgh");
    }

    #[test]
    fn toggle_key_state_cycles_default_shift_symbol() {
        // `CycleSymbols` (`:933-950`).
        let mut state = KeyboardState::new(&Request {
            label: String::new(),
            initial: "x".into(),
            allow_empty: false,
            max_length: 20,
        });
        assert_eq!(state.key_state, KeyState::Default);
        state.toggle_key_state();
        assert_eq!(state.key_state, KeyState::Shift);
        assert!(state.shift_latched);
        state.toggle_key_state();
        assert_eq!(state.key_state, KeyState::Symbol);
        assert!(!state.shift_latched);
        state.toggle_key_state();
        assert_eq!(state.key_state, KeyState::Default);
        // `set_key_state` is the direct route (`SetKeyState`, `:535-583`).
        state.set_key_state(KeyState::Symbol);
        assert_eq!(state.key_state, KeyState::Symbol);
        assert!(!state.shift_latched);
    }

    #[test]
    fn click_at_maps_every_visible_key_centre() {
        let l = layout(SCREEN, true);
        let mut state = KeyboardState::new(&Request {
            label: String::new(),
            initial: String::new(),
            allow_empty: true,
            max_length: 20,
        });
        let centre = |rect: [f32; 4]| Vec2::new(rect[0] + rect[2] * 0.5, rect[1] + rect[3] * 0.5);
        // The grid's corners and a middle key (`:219-227`) — the letters read
        // the live `KeyState`, so pin Default first.
        state.set_key_state(KeyState::Default);
        assert_eq!(
            click_at(&state, &l, true, centre(key_rect(l.grid_origin, 0, 0, 1))),
            Click::Char('1')
        );
        assert_eq!(
            click_at(&state, &l, true, centre(key_rect(l.grid_origin, 9, 3, 1))),
            Click::Char('?')
        );
        assert_eq!(
            click_at(&state, &l, true, centre(key_rect(l.grid_origin, 4, 2, 1))),
            Click::Char('g')
        );
        // The labels follow the live `KeyState` (`:574-577`).
        state.set_key_state(KeyState::Shift);
        assert_eq!(
            click_at(&state, &l, true, centre(key_rect(l.grid_origin, 0, 1, 1))),
            Click::Char('Q')
        );
        state.set_key_state(KeyState::Symbol);
        assert_eq!(
            click_at(&state, &l, true, centre(key_rect(l.grid_origin, 3, 1, 1))),
            Click::Char('$')
        );
        state.set_key_state(KeyState::Default);
        // The bottom row (`:228`, `:263`, `:452`, `:391`).
        assert_eq!(click_at(&state, &l, true, centre(key_rect(l.grid_origin, 0, 4, 1))), Click::Shift);
        assert_eq!(click_at(&state, &l, true, centre(key_rect(l.grid_origin, 1, 4, 1))), Click::Symbol);
        assert_eq!(click_at(&state, &l, true, centre(key_rect(l.grid_origin, 2, 4, 6))), Click::Space);
        assert_eq!(click_at(&state, &l, true, centre(key_rect(l.grid_origin, 8, 4, 2))), Click::Backspace);
        // The small pair (`:313`, `:326`) — Submit reports even when invalid;
        // the handler re-checks (`:358-365`).
        assert_eq!(click_at(&state, &l, true, centre(l.submit)), Click::Submit);
        assert_eq!(click_at(&state, &l, true, centre(l.cancel)), Click::Cancel);
        assert_eq!(click_at(&state, &l, true, Vec2::new(100.0, 100.0)), Click::None);

        // The desktop layer: only the big pair answers (`:366`, `:390`); the
        // hidden grid is dead.
        let d = layout(SCREEN, false);
        assert_eq!(click_at(&state, &d, false, centre(key_rect(d.grid_origin, 0, 0, 1))), Click::None);
        assert_eq!(click_at(&state, &d, false, centre(d.submit)), Click::Submit);
        assert_eq!(click_at(&state, &d, false, centre(d.cancel)), Click::Cancel);
        assert_eq!(click_at(&state, &d, false, Vec2::new(100.0, 100.0)), Click::None);
    }

    #[test]
    fn panel_bg_truncates_to_the_xna_bytes() {
        // `new Color(23, 33, 69) * 0.7f` (`:206`).
        assert_eq!(PANEL_BG, color::mul([23, 33, 69, 255], 0.7));
        assert_eq!(PANEL_BG, [16, 23, 48, 178]);
    }
}
