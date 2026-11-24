//! The text-box element — W4-d83 (research/18b row 54): `UITextBox`
//! (`GameContent/UI/Elements/UITextBox.cs:9-111`, an *internal*
//! `UITextPanel<string>` subclass). The on-screen keyboard's write/backspace
//! halves are landed (`keyboard.rs:183-246`, the caret draw `:764-808`);
//! this module carries the element's own state machine: the max-length
//! trim + cursor clamp, the cursor walk, the draw's cursor-to-end pin and
//! the 40-frame input ticker with its caret offsets. The IME composition
//! string is the caller's (the port has no IME service).

use super::search_bar::{trim_last_character, trim_user_string};

/// `UITextBox` (`UITextBox.cs:9-111`), research/18b row 54 (W4-d83).
/// Lengths and the cursor count **UTF-16 code units** like C# `string.Length`.
#[derive(Clone, Debug, PartialEq)]
pub struct TextBox {
    /// The panel's `Text` (the base `UITextPanel<string>`).
    pub text: String,
    /// `_cursor` (`:11`) — UTF-16 index into `text`.
    pub cursor: usize,
    /// `_frameCount` (`:13`) — the ticker's counter.
    frame_count: i32,
    /// `_maxLength = 20` (`:15`) — the ctor default, unlike the search
    /// bar's 50 (`UISearchBar.cs:63` sets it explicitly).
    pub max_length: usize,
    /// `ShowInputTicker = true` (`:17`).
    pub show_input_ticker: bool,
    /// `HideSelf` (`:19`) — the draw's early return (`:82-85`).
    pub hide_self: bool,
    /// `_isLarge` — the base panel's font pick, read by the caret offsets.
    pub large: bool,
    /// `_textScale` — the base panel's scale.
    pub text_scale: f32,
}

impl TextBox {
    /// The ctor (`:9`): the text goes through the `SetText` trim, cursor 0.
    pub fn new(text: &str, text_scale: f32, large: bool) -> Self {
        let mut b = Self {
            text: String::new(),
            cursor: 0,
            frame_count: 0,
            max_length: 20,
            show_input_ticker: true,
            hide_self: false,
            large,
            text_scale,
        };
        b.set_text(text);
        b
    }

    fn utf16_len(s: &str) -> usize {
        s.encode_utf16().count()
    }

    /// `SetText` (`:44-49`): trim to `_maxLength` (null reads `""`), then
    /// clamp the cursor to the new length. The base's re-measure is the
    /// caller's.
    pub fn set_text(&mut self, text: &str) {
        self.text = trim_user_string(text, self.max_length); // `:46`
        self.cursor = self.cursor.min(Self::utf16_len(&self.text)); // `:48`
    }

    /// `SetTextMaxLength` (`:51-54`): a plain set — no re-trim until the
    /// next `SetText` (the search bar's setter re-trims through its box).
    pub fn set_text_max_length(&mut self, max_length: usize) {
        self.max_length = max_length;
    }

    /// `Write` (`:38-42`): insert at the cursor, advance by the inserted
    /// length — **through `SetText`**, so the insert trims to the cap and
    /// the cursor clamps (`:40` calls the override, not the base).
    pub fn write(&mut self, text: &str) {
        let units: Vec<u16> = self.text.encode_utf16().collect();
        let ins: Vec<u16> = text.encode_utf16().collect();
        let at = self.cursor.min(units.len());
        let mut joined = units[..at].to_vec();
        joined.extend_from_slice(&ins);
        joined.extend_from_slice(&units[at..]);
        let joined = String::from_utf16_lossy(&joined);
        self.set_text(&joined); // `:40` the trimming override
        self.cursor += Self::utf16_len(text); // `:41` — past the clamp, like C#
    }

    /// `Backspace` (`:56-62`): only when the cursor is non-zero, drop the
    /// **last** character of the whole text (not the one at the cursor —
    /// the vanilla quirk) through the trimming `SetText`.
    pub fn backspace(&mut self) {
        if self.cursor != 0 {
            let trimmed = trim_last_character(&self.text);
            self.set_text(&trimmed);
        }
    }

    /// `CursorLeft` (`:64-70`) / `CursorRight` (`:72-78`): bounded walks.
    pub fn cursor_left(&mut self) {
        if self.cursor != 0 {
            self.cursor -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor < Self::utf16_len(&self.text) {
            self.cursor += 1;
        }
    }

    /// `DrawSelf`'s state half (`:80-109`): `HideSelf` skips everything;
    /// otherwise the cursor **pins to the text end** every draw (`:86`),
    /// then the ticker advances: `++_frameCount %= 40`, caret visible while
    /// `<= 20` (`:99-100`) — 21 of every 40 frames. Returns `None` when
    /// hidden, else `Some(caret_visible)`.
    pub fn draw_tick(&mut self) -> Option<bool> {
        if self.hide_self {
            return None; // `:82-85`
        }
        self.cursor = Self::utf16_len(&self.text); // `:86`
        if !self.show_input_ticker {
            return Some(false); // `:88-91` — no counter advance either
        }
        self.frame_count += 1; // `:99`
        self.frame_count %= 40; // `:100`
        Some(self.frame_count <= 20)
    }

    /// The caret's draw offsets (`:102-107`), added to the panel's
    /// `TextDrawPosition`: x rides the measured prefix width (`prefix_w` =
    /// `Font.MeasureString(Text.Substring(0, _cursor)).X * TextScale`) plus
    /// `6 - (large ? 8 : 4) * scale`; the large font also shifts y by
    /// `2 * scale`. The caret draws `"|"` in `TextColor` (`:108`).
    pub fn caret_offset(&self, prefix_w: f32) -> [f32; 2] {
        [
            prefix_w + 6.0 - (if self.large { 8.0 } else { 4.0 }) * self.text_scale,
            if self.large { 2.0 * self.text_scale } else { 0.0 },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SetText`/`Write` (`UITextBox.cs:38-49`): the 20-unit default cap,
    /// the cursor clamp, and Write's insert-at-cursor through the trim.
    #[test]
    fn the_box_trims_to_twenty_and_clamps_the_cursor() {
        let mut b = TextBox::new("hello", 1.0, false);
        assert_eq!(b.max_length, 20);
        assert_eq!(b.text, "hello");
        b.cursor = 5;
        b.set_text("ab");
        assert_eq!(b.cursor, 2, "the clamp (`:48`)");
        b.write("cd");
        assert_eq!(b.text, "abcd");
        assert_eq!(b.cursor, 4);
        let long = "x".repeat(30);
        b.set_text(&long);
        assert_eq!(b.text.len(), 20);
        // Write past the cap: the text trims but the cursor advances raw
        // (`:41` runs after the trimming SetText).
        b.write("yy");
        assert_eq!(b.text.len(), 20);
        assert_eq!(b.cursor, 22, "the C# cursor is not re-clamped by Write");
    }

    /// `Backspace`/cursor walks (`:56-78`): backspace drops the LAST char
    /// even mid-text (the vanilla quirk), zero cursor blocks it; the walks
    /// bound at 0 and the UTF-16 length.
    #[test]
    fn backspace_drops_the_tail_and_the_walks_bound() {
        let mut b = TextBox::new("abcd", 1.0, false);
        b.cursor = 2;
        b.backspace();
        assert_eq!(b.text, "abc", "the tail goes, not text[cursor-1] (`:60`)");
        assert_eq!(b.cursor, 2, "the clamp keeps 2 <= 3");
        b.cursor = 0;
        b.backspace();
        assert_eq!(b.text, "abc", "zero cursor: no-op (`:58`)");
        b.cursor_left();
        assert_eq!(b.cursor, 0);
        for _ in 0..10 {
            b.cursor_right();
        }
        assert_eq!(b.cursor, 3, "bounded at the length (`:74`)");
    }

    /// The ticker (`:99-108`): 21 visible frames per 40 (counts 1..=20 and
    /// 0), the caret offsets ride the scale, HideSelf skips the pin.
    #[test]
    fn the_ticker_shows_21_of_40_and_the_caret_rides_the_scale() {
        let mut b = TextBox::new("ab", 0.8, false);
        b.cursor = 0;
        let mut visible = 0;
        for _ in 0..40 {
            if b.draw_tick() == Some(true) {
                visible += 1;
            }
        }
        assert_eq!(visible, 21, "counts 1..20 plus the 0 wrap (`:100`)");
        assert_eq!(b.cursor, 2, "every draw pins to the end (`:86`)");
        // Small: 6 - 4*0.8 = 2.8; large: 6 - 8*0.8 = -0.4, y +1.6.
        assert_eq!(b.caret_offset(10.0), [12.8, 0.0]);
        b.large = true;
        let off = b.caret_offset(10.0);
        assert!((off[0] - 9.6).abs() < 1e-5);
        assert!((off[1] - 1.6).abs() < 1e-5);
        b.hide_self = true;
        assert_eq!(b.draw_tick(), None);
        b.hide_self = false;
        b.show_input_ticker = false;
        assert_eq!(b.draw_tick(), Some(false));
    }
}
