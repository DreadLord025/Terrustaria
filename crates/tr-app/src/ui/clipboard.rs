//! The OS text clipboard behind the character creator's Copy/Paste buttons —
//! the port of `Platform.Get<IClipboard>().Value` (`Terraria/Platform.cs`),
//! which `UICharacterCreation` reads and writes at
//! `UICharacterCreation.cs:1334` (copy hex), `:1341` (paste hex), `:1411`
//! (copy template) and `:1419` (paste template).
//!
//! The C# routes this through the `Platform` service locator; the front-end is
//! the only caller, so the port keeps it as a stateless free-function bridge
//! rather than parking a service in `App`. Backed by `arboard`: clipboard access
//! is process-wide OS state, not game state, so nothing here participates in the
//! deterministic step or the frame cache. Failure (no clipboard, no text) is
//! silent, like every other platform-service path in the port.

use arboard::Clipboard;

/// `IClipboard.Value`'s getter (`Platform.Get<IClipboard>().Value`): the current
/// clipboard text, or `None` when the clipboard is unavailable or empty.
pub fn get_text() -> Option<String> {
    Clipboard::new().ok().and_then(|mut c| c.get_text().ok())
}

/// `IClipboard.Value`'s setter: replaces the clipboard text. A missing clipboard
/// is a silent no-op.
pub fn set_text(text: &str) {
    if let Ok(mut c) = Clipboard::new() {
        let _ = c.set_text(text.to_owned());
    }
}
