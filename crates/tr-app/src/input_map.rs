//! `Main.cUp`..`Main.cMount` and `Main.cMapStyle`..`Main.cMapAlphaDown`: the
//! keyboard bind table the settings screens edit and the input pass reads.
//!
//! The vanilla keeps the 14 gameplay binds and the 6 map binds as plain
//! strings (`Main.cs:67163-67176` / `:67190-67195`) and hands them to
//! `PlayerInput.CurrentProfile`'s `KeyStatus` map (`Main.cs:4602-4625`), so a
//! rebind is just a string swap. The port keeps the same shape: the strings
//! live in [`crate::settings::Settings`] (persisted under the vanilla
//! `config.json` names, which `Main.LoadSettings` reads back at
//! `Main.cs:4582-4600`), and every input site asks this module whether a
//! `KeyCode` is bound to an [`Action`].
//!
//! The names are XNA `Microsoft.Xna.Framework.Input.Keys` spellings ("W",
//! "Space", "LeftShift", "Add", "PageUp", …), not winit's, because that is
//! what the vanilla writes into `config.json` and what the settings screen
//! prints.

use winit::keyboard::KeyCode;

/// The 14 gameplay rows of the menuMode 27 list, in the vanilla's own **UI**
/// order (`Main.cs:50505-50520`'s `array11` and the `Main.setKey` chain at
/// `:50577-50628`): `cUp, cDown, cLeft, cRight, cJump, cThrowItem, cInv,
/// cHeal, cMana, cBuff, cHook, cTorch, cSmart, cMount`.
///
/// That is *not* the declaration order of the static seed
/// (`Main.cs:67163-67176`, which assigns `cUp, cLeft, cDown, cRight`); the
/// screen's row indices — and therefore `Main.setKey` — follow `array11`.
pub const BIND_COUNT: usize = 14;

/// The 6 map rows of the menuMode 272727 list (`Main.cs:67190-67195`):
/// `cMapStyle, cMapFull, cMapZoomIn, cMapZoomOut, cMapAlphaUp, cMapAlphaDown`.
pub const MAP_BIND_COUNT: usize = 6;

/// One gameplay bind row. The discriminant is the row index in
/// [`default_binds`], so `Action::ALL[i]` is the action of `binds[i]`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// `Main.cUp` — `Player.controlUp`.
    Up,
    /// `Main.cLeft` — `Player.controlLeft`.
    Left,
    /// `Main.cDown` — `Player.controlDown`.
    Down,
    /// `Main.cRight` — `Player.controlRight`.
    Right,
    /// `Main.cJump` — `Player.controlJump`.
    Jump,
    /// `Main.cThrowItem` — `Player.controlThrow`.
    Throw,
    /// `Main.cHeal` — `Player.controlQuickHeal`.
    QuickHeal,
    /// `Main.cMana` — `Player.controlQuickMana`.
    QuickMana,
    /// `Main.cBuff` — `Player.controlQuickBuff`.
    QuickBuff,
    /// `Main.cHook` — `Player.controlHook`.
    Hook,
    /// `Main.cTorch` — `Player.controlTorch` (auto-select).
    Torch,
    /// `Main.cInv` — `Player.controlInv`.
    Inventory,
    /// `Main.cSmart` — `Player.controlSmart`.
    Smart,
    /// `Main.cMount` — `Player.controlMount`.
    Mount,
}

impl Action {
    /// The row order of the menuMode 27 list (`Main.cs:50505-50520`) — the
    /// order the screen prints and `Main.setKey` writes back.
    pub const ALL: [Action; BIND_COUNT] = [
        Action::Up,
        Action::Down,
        Action::Left,
        Action::Right,
        Action::Jump,
        Action::Throw,
        Action::Inventory,
        Action::QuickHeal,
        Action::QuickMana,
        Action::QuickBuff,
        Action::Hook,
        Action::Torch,
        Action::Smart,
        Action::Mount,
    ];

    /// The `config.json` key `Main.LoadSettings` reads this row from
    /// (`Main.cs:4582-4595`), in the same order as [`Action::ALL`].
    pub const CONFIG_KEYS: [&'static str; BIND_COUNT] = [
        "KeyUp",
        "KeyDown",
        "KeyLeft",
        "KeyRight",
        "KeyJump",
        "KeyThrowItem",
        "KeyInventory",
        "KeyQuickHeal",
        "KeyQuickMana",
        "KeyQuickBuff",
        "KeyUseHook",
        "KeyAutoSelect",
        "KeySmartCursor",
        "KeyMount",
    ];

    /// The row index, i.e. the position in [`default_binds`].
    pub fn index(self) -> usize {
        Action::ALL.iter().position(|a| *a == self).expect("every action is in ALL")
    }
}

/// One map bind row, in the menuMode 272727 order (`Main.cs:67190-67195`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapAction {
    /// `Main.cMapStyle` — the map's frame style cycle.
    Style,
    /// `Main.cMapFull` — `Main.TryOpeningFullscreenMap`.
    Full,
    /// `Main.cMapZoomIn`.
    ZoomIn,
    /// `Main.cMapZoomOut`.
    ZoomOut,
    /// `Main.cMapAlphaUp`.
    AlphaUp,
    /// `Main.cMapAlphaDown`.
    AlphaDown,
}

impl MapAction {
    pub const ALL: [MapAction; MAP_BIND_COUNT] = [
        MapAction::Style,
        MapAction::Full,
        MapAction::ZoomIn,
        MapAction::ZoomOut,
        MapAction::AlphaUp,
        MapAction::AlphaDown,
    ];

    /// The `config.json` keys (`Main.cs:4596-4600`).
    pub const CONFIG_KEYS: [&'static str; MAP_BIND_COUNT] = [
        "KeyMapStyle",
        "KeyFullscreenMap",
        "KeyMapZoomIn",
        "KeyMapZoomOut",
        "KeyMapAlphaUp",
        "KeyMapAlphaDown",
    ];

    pub fn index(self) -> usize {
        MapAction::ALL.iter().position(|a| *a == self).expect("every map action is in ALL")
    }
}

/// `Main.cUp`..`Main.cMount`'s static seed (`Main.cs:67163-67176`), arranged in
/// the menuMode 27 row order (`array11`, `Main.cs:50505-50520`): `W, S, A, D`
/// — the source assigns `cUp = "W"`, `cLeft = "A"`, `cDown = "S"`,
/// `cRight = "D"`, and the screen lists Up, Down, Left, Right.
pub fn default_binds() -> [String; BIND_COUNT] {
    [
        "W",           // cUp
        "S",           // cDown
        "A",           // cLeft
        "D",           // cRight
        "Space",       // cJump
        "T",           // cThrowItem
        "Escape",      // cInv
        "H",           // cHeal
        "J",           // cMana
        "B",           // cBuff
        "E",           // cHook
        "LeftShift",   // cTorch
        "LeftControl", // cSmart
        "R",           // cMount
    ]
    .map(String::from)
}

/// `Main.cMapStyle`..`Main.cMapAlphaDown`'s static seed (`Main.cs:67190-67195`).
pub fn default_map_binds() -> [String; MAP_BIND_COUNT] {
    ["Tab", "M", "Add", "Subtract", "PageUp", "PageDown"].map(String::from)
}

/// The XNA `Keys` name of a winit `KeyCode`, or `None` for a key the vanilla
/// has no name for. Only the names the settings screen can print are covered;
/// the rest fall back to the winit debug spelling so a rebind still round-trips
/// through [`parse_key`].
pub fn key_name(code: KeyCode) -> String {
    use KeyCode::*;
    let s = match code {
        KeyA => "A",
        KeyB => "B",
        KeyC => "C",
        KeyD => "D",
        KeyE => "E",
        KeyF => "F",
        KeyG => "G",
        KeyH => "H",
        KeyI => "I",
        KeyJ => "J",
        KeyK => "K",
        KeyL => "L",
        KeyM => "M",
        KeyN => "N",
        KeyO => "O",
        KeyP => "P",
        KeyQ => "Q",
        KeyR => "R",
        KeyS => "S",
        KeyT => "T",
        KeyU => "U",
        KeyV => "V",
        KeyW => "W",
        KeyX => "X",
        KeyY => "Y",
        KeyZ => "Z",
        Digit0 => "D0",
        Digit1 => "D1",
        Digit2 => "D2",
        Digit3 => "D3",
        Digit4 => "D4",
        Digit5 => "D5",
        Digit6 => "D6",
        Digit7 => "D7",
        Digit8 => "D8",
        Digit9 => "D9",
        F1 => "F1",
        F2 => "F2",
        F3 => "F3",
        F4 => "F4",
        F5 => "F5",
        F6 => "F6",
        F7 => "F7",
        F8 => "F8",
        F9 => "F9",
        F10 => "F10",
        F11 => "F11",
        F12 => "F12",
        Space => "Space",
        Escape => "Escape",
        Enter => "Enter",
        Tab => "Tab",
        Backspace => "Back",
        ShiftLeft => "LeftShift",
        ShiftRight => "RightShift",
        ControlLeft => "LeftControl",
        ControlRight => "RightControl",
        AltLeft => "LeftAlt",
        AltRight => "RightAlt",
        ArrowUp => "Up",
        ArrowDown => "Down",
        ArrowLeft => "Left",
        ArrowRight => "Right",
        PageUp => "PageUp",
        PageDown => "PageDown",
        Home => "Home",
        End => "End",
        Insert => "Insert",
        Delete => "Delete",
        NumpadAdd => "Add",
        NumpadSubtract => "Subtract",
        NumpadMultiply => "Multiply",
        NumpadDivide => "Divide",
        NumpadEnter => "Enter",
        Numpad0 => "NumPad0",
        Numpad1 => "NumPad1",
        Numpad2 => "NumPad2",
        Numpad3 => "NumPad3",
        Numpad4 => "NumPad4",
        Numpad5 => "NumPad5",
        Numpad6 => "NumPad6",
        Numpad7 => "NumPad7",
        Numpad8 => "NumPad8",
        Numpad9 => "NumPad9",
        Semicolon => "OemSemicolon",
        Equal => "OemPlus",
        Comma => "OemComma",
        Minus => "OemMinus",
        Period => "OemPeriod",
        Slash => "OemQuestion",
        Backquote => "OemTilde",
        BracketLeft => "OemOpenBrackets",
        Backslash => "OemPipe",
        BracketRight => "OemCloseBrackets",
        Quote => "OemQuotes",
        _ => return format!("{code:?}"),
    };
    s.to_string()
}

/// The inverse of [`key_name`]. Unknown names return `None`, which the settings
/// parser treats as "keep the default" (the vanilla's `Get` leaves the field
/// untouched when the key is absent, `Main.cs:4582`).
pub fn parse_key(name: &str) -> Option<KeyCode> {
    use KeyCode::*;
    let n = name.trim();
    // The single-letter rows are the common case.
    if n.len() == 1 {
        let c = n.chars().next()?.to_ascii_uppercase();
        if c.is_ascii_uppercase() {
            return Some(match c {
                'A' => KeyA,
                'B' => KeyB,
                'C' => KeyC,
                'D' => KeyD,
                'E' => KeyE,
                'F' => KeyF,
                'G' => KeyG,
                'H' => KeyH,
                'I' => KeyI,
                'J' => KeyJ,
                'K' => KeyK,
                'L' => KeyL,
                'M' => KeyM,
                'N' => KeyN,
                'O' => KeyO,
                'P' => KeyP,
                'Q' => KeyQ,
                'R' => KeyR,
                'S' => KeyS,
                'T' => KeyT,
                'U' => KeyU,
                'V' => KeyV,
                'W' => KeyW,
                'X' => KeyX,
                'Y' => KeyY,
                _ => KeyZ,
            });
        }
    }
    // `D0`..`D9` and `F1`..`F12` are the two numbered families.
    if let Some(d) = n.strip_prefix('D').and_then(|d| d.parse::<u8>().ok()) {
        return Some(match d {
            0 => Digit0,
            1 => Digit1,
            2 => Digit2,
            3 => Digit3,
            4 => Digit4,
            5 => Digit5,
            6 => Digit6,
            7 => Digit7,
            8 => Digit8,
            9 => Digit9,
            _ => return None,
        });
    }
    if let Some(f) = n.strip_prefix('F').and_then(|f| f.parse::<u8>().ok()) {
        return Some(match f {
            1 => F1,
            2 => F2,
            3 => F3,
            4 => F4,
            5 => F5,
            6 => F6,
            7 => F7,
            8 => F8,
            9 => F9,
            10 => F10,
            11 => F11,
            12 => F12,
            _ => return None,
        });
    }
    Some(match n {
        "Space" => Space,
        "Escape" => Escape,
        "Enter" => Enter,
        "Tab" => Tab,
        "Back" => Backspace,
        "LeftShift" => ShiftLeft,
        "RightShift" => ShiftRight,
        "LeftControl" => ControlLeft,
        "RightControl" => ControlRight,
        "LeftAlt" => AltLeft,
        "RightAlt" => AltRight,
        "Up" => ArrowUp,
        "Down" => ArrowDown,
        "Left" => ArrowLeft,
        "Right" => ArrowRight,
        "PageUp" => PageUp,
        "PageDown" => PageDown,
        "Home" => Home,
        "End" => End,
        "Insert" => Insert,
        "Delete" => Delete,
        "Add" => NumpadAdd,
        "Subtract" => NumpadSubtract,
        "Multiply" => NumpadMultiply,
        "Divide" => NumpadDivide,
        "NumPad0" => Numpad0,
        "NumPad1" => Numpad1,
        "NumPad2" => Numpad2,
        "NumPad3" => Numpad3,
        "NumPad4" => Numpad4,
        "NumPad5" => Numpad5,
        "NumPad6" => Numpad6,
        "NumPad7" => Numpad7,
        "NumPad8" => Numpad8,
        "NumPad9" => Numpad9,
        "OemSemicolon" => Semicolon,
        "OemPlus" => Equal,
        "OemComma" => Comma,
        "OemMinus" => Minus,
        "OemPeriod" => Period,
        "OemQuestion" => Slash,
        "OemTilde" => Backquote,
        "OemOpenBrackets" => BracketLeft,
        "OemPipe" => Backslash,
        "OemCloseBrackets" => BracketRight,
        "OemQuotes" => Quote,
        _ => return None,
    })
}

/// The gameplay action bound to `code`, if any. A row whose string does not
/// parse (a hand-edited `config.json`) simply binds nothing, like the vanilla's
/// `KeyStatus` lookup missing the name.
pub fn action_for(binds: &[String; BIND_COUNT], code: KeyCode) -> Option<Action> {
    Action::ALL
        .iter()
        .copied()
        .find(|a| binds[a.index()].as_str() == key_name(code))
}

/// The map action bound to `code`, if any.
pub fn map_action_for(binds: &[String; MAP_BIND_COUNT], code: KeyCode) -> Option<MapAction> {
    MapAction::ALL
        .iter()
        .copied()
        .find(|a| binds[a.index()].as_str() == key_name(code))
}

/// `Player.controlUp`-style hold test: is `code` the row's key?
pub fn is_bound(binds: &[String; BIND_COUNT], action: Action, code: KeyCode) -> bool {
    binds[action.index()].as_str() == key_name(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default table is the vanilla's own seed, in the vanilla's own
    /// declaration order (`Main.cs:67163-67176`) — `cUp = "W"`, `cLeft = "A"`,
    /// `cDown = "S"`, `cRight = "D"`, and `cSmart` is **LeftControl** while
    /// `cTorch` is **LeftShift** (the port had them swapped).
    #[test]
    fn the_default_binds_are_the_c_sharp_seed() {
        let b = default_binds();
        // The menuMode 27 row order (`Main.cs:50505-50520`): Up, Down, Left,
        // Right — so the table reads `W, S, A, D`.
        assert_eq!(b[0], "W");
        assert_eq!(b[1], "S");
        assert_eq!(b[2], "A");
        assert_eq!(b[3], "D");
        assert_eq!(b[Action::Up.index()], "W");
        assert_eq!(b[Action::Left.index()], "A");
        assert_eq!(b[Action::Down.index()], "S");
        assert_eq!(b[Action::Right.index()], "D");
        assert_eq!(b[Action::Jump.index()], "Space");
        assert_eq!(b[Action::Throw.index()], "T");
        assert_eq!(b[Action::QuickHeal.index()], "H");
        assert_eq!(b[Action::QuickMana.index()], "J");
        assert_eq!(b[Action::QuickBuff.index()], "B");
        assert_eq!(b[Action::Hook.index()], "E");
        assert_eq!(b[Action::Torch.index()], "LeftShift");
        assert_eq!(b[Action::Inventory.index()], "Escape");
        assert_eq!(b[Action::Smart.index()], "LeftControl");
        assert_eq!(b[Action::Mount.index()], "R");
        // The map rows (`Main.cs:67190-67195`).
        let m = default_map_binds();
        assert_eq!(m[MapAction::Style.index()], "Tab");
        assert_eq!(m[MapAction::Full.index()], "M");
        assert_eq!(m[MapAction::ZoomIn.index()], "Add");
        assert_eq!(m[MapAction::ZoomOut.index()], "Subtract");
        assert_eq!(m[MapAction::AlphaUp.index()], "PageUp");
        assert_eq!(m[MapAction::AlphaDown.index()], "PageDown");
    }

    /// Every default name parses back to the key it names, and the name of that
    /// key is the same string — the round trip the settings screen relies on.
    #[test]
    fn every_default_name_round_trips() {
        for name in default_binds().iter().chain(default_map_binds().iter()) {
            let code = parse_key(name).unwrap_or_else(|| panic!("{name} parses"));
            assert_eq!(&key_name(code), name, "{name} round-trips");
        }
    }

    /// The lookup is by name, so a rebind moves the action with it.
    #[test]
    fn a_rebind_moves_the_action() {
        let mut b = default_binds();
        assert_eq!(action_for(&b, KeyCode::KeyM), None, "M is not a gameplay row");
        b[Action::Inventory.index()] = "M".into();
        assert_eq!(action_for(&b, KeyCode::KeyM), Some(Action::Inventory));
        assert_eq!(action_for(&b, KeyCode::Escape), None, "Escape is free now");
        // The map table is separate: M is still the fullscreen map.
        let m = default_map_binds();
        assert_eq!(map_action_for(&m, KeyCode::KeyM), Some(MapAction::Full));
    }

    /// An unparseable row binds nothing rather than panicking.
    #[test]
    fn a_garbage_row_binds_nothing() {
        let mut b = default_binds();
        b[Action::Jump.index()] = "NotAKey".into();
        assert_eq!(action_for(&b, KeyCode::Space), None);
        assert_eq!(parse_key("NotAKey"), None);
    }
}
