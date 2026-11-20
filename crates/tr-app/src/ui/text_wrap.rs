//! Word wrapping — `ReLogic.Text.WrappedTextBuilder`
//! (`WrappedTextBuilder.cs:1-171`) + `StringReaderWrapExtension`
//! (`StringReaderWrapExtension.cs:1-132`), reached through
//! `DynamicSpriteFont.CreateWrappedText` (`DynamicSpriteFont.cs:329-341`)
//! and `Utils.WordwrapString` (`Utils.cs:860-867`). The research/37 gap:
//! before this module no `wrap` helper existed in `crates/`, so long
//! labels/tooltips could not wrap like the original.
//!
//! The culture rules ride `Language.ActiveCulture.CultureInfo` — the port
//! maps it to [`GameCulture`] (`tr-core/localization/culture.rs`); only the
//! two Chinese cultures take the `BreaksBetweenMostGlyphs` branch
//! (`StringReaderWrapExtension.cs:44-51`).
//!
//! Glyph metrics: `GlyphMetrics.FromKerningData(Kerning.X/Y/Z)`
//! (`GlyphMetrics.cs:24-27`) — `KernedWidth = X+Y+Z`,
//! `KernedWidthOnNewLine = max(0,X)+Y+Z` (`:13-15`). The port measures
//! through the retail `Mouse_Text` sheet ([`assets::UiFont`]); the bitmap
//! fallback estimates 8 px per char (the same recorded stand-in
//! `npc_chat::panel_lines` carries — docs/ANALYSIS.md).

use tr_core::localization::GameCulture;

use crate::assets::UiFont;

/// `GlyphMetrics.KernedWidth` (`GlyphMetrics.cs:13`): X+Y+Z.
fn kerned_width(font: Option<&UiFont>, c: char) -> f32 {
    match font.and_then(|f| f.glyph(c)) {
        Some(g) => g.kerning[0] + g.kerning[1] + g.kerning[2],
        None => 8.0,
    }
}

/// `GlyphMetrics.KernedWidthOnNewLine` (`GlyphMetrics.cs:15`):
/// `max(0, X) + Y + Z`.
fn kerned_width_new_line(font: Option<&UiFont>, c: char) -> f32 {
    match font.and_then(|f| f.glyph(c)) {
        Some(g) => g.kerning[0].max(0.0) + g.kerning[1] + g.kerning[2],
        None => 8.0,
    }
}

/// `DynamicSpriteFont.CharacterSpacing` (`DynamicSpriteFont.cs:49`) — the
/// bitmap fallback has no spacing.
fn char_spacing(font: Option<&UiFont>) -> f32 {
    font.map_or(0.0, |f| f.spacing)
}

/// `StringReaderWrapExtension.BreaksBetweenMostGlyphs` (`:44-51`): the
/// zh-Hans/zh-Hant LCIDs only.
fn breaks_between_most_glyphs(culture: GameCulture) -> bool {
    matches!(
        culture,
        GameCulture::Chinese | GameCulture::ChineseTraditional
    )
}

/// `StringReaderWrapExtension.IsIgnoredCharacter` (`:53-60`): control
/// characters below ' ' except '\n'.
fn is_ignored(c: char) -> bool {
    c < ' ' && c != '\n'
}

/// `StringReaderWrapExtension.GetModeForCharacter` (`:79-97`) — the four
/// scan modes.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ScanMode {
    Space,
    NewLine,
    Word,
    None,
}

fn mode_for(c: char) -> ScanMode {
    if is_ignored(c) {
        return ScanMode::None;
    }
    match c {
        '\n' => ScanMode::NewLine,
        ' ' | '\u{2009}' | '\u{200a}' | '\u{200b}' => ScanMode::Space,
        _ => ScanMode::Word,
    }
}

/// `InvalidCharactersForLineStart` (`StringReaderWrapExtension.cs:32`) —
/// glyphs that may not START a line (closing punctuation &c.).
const INVALID_LINE_START: &str = "!%),.:;?]}¢°·'\"†‡›℃∶、。〃〆〕〗〞﹚﹜！＂％＇），．：；？！］｝～ \n!),.:;?]}¢·–— '\"• 、。〆〞〕〉》」︰︱︲\u{fe33}﹐﹑﹒\u{fe53}﹔﹕﹖﹘﹚﹜！），．：；？︶︸︺︼︾﹀﹂﹗］｜｝､";

/// `InvalidCharactersForLineEnd` (`:34`) — glyphs that may not END a line
/// (opening punctuation &c.).
const INVALID_LINE_END: &str =
    "$(£¥·'\"〈《「『【〔〖〝﹙﹛＄（．［｛￡￥([{£¥'\"‵〈《「『〔〝\u{fe34}﹙﹛（｛︵︷︹︻︽︿﹁﹃\u{fe4f}";

/// `WordTerminators` (`:36`) — the CJK sentence ends that allow a break in
/// the non-Chinese branch.
const WORD_TERMINATORS: &str = "！，。、：？";

/// `Numeric` (`:38`) — digit runs (with ',' and '.') stay together in the
/// Chinese branch.
const NUMERIC: &str = "0123456789,.";

/// `StringReaderWrapExtension.CanBreakBetween` (`:62-77`).
fn can_break_between(prev: char, next: char, culture: GameCulture) -> bool {
    if breaks_between_most_glyphs(culture) {
        if NUMERIC.contains(prev) && NUMERIC.contains(next) {
            return false;
        }
        if !INVALID_LINE_END.contains(prev) {
            return !INVALID_LINE_START.contains(next);
        }
        return false;
    }
    WORD_TERMINATORS.contains(prev)
}

/// `StringReaderWrapExtension.ReadUntilBreakable` (`:99-131`): consume the
/// next non-breaking token — a run of one scan mode, split further where
/// `CanBreakBetween` allows. Ignored characters are skipped mid-scan; the
/// caller handles '\n' itself (the `Peek() == 10` arm, `Build` `:157-161`).
fn read_until_breakable(chars: &[char], pos: &mut usize, culture: GameCulture) -> String {
    let mut out = String::new();
    let mut c = chars[*pos];
    let mut mode = ScanMode::None;
    // `reader.Peek() > 0` (`:104`): a NUL ends the scan like the stream end.
    while *pos < chars.len() && chars[*pos] != '\0' {
        if is_ignored(chars[*pos]) {
            *pos += 1;
            continue;
        }
        let prev = c;
        c = chars[*pos];
        let prev_mode = mode;
        mode = mode_for(c);
        if !out.is_empty() && prev_mode != mode {
            return out;
        }
        if out.is_empty() {
            out.push(chars[*pos]);
            *pos += 1;
            continue;
        }
        if can_break_between(prev, c, culture) {
            return out;
        }
        out.push(chars[*pos]);
        *pos += 1;
    }
    out
}

/// `WrappedTextBuilder.NonBreakingText` (`WrappedTextBuilder.cs:9-48`):
/// the token's plain width (spacing between every char), its first-char
/// new-line width delta, and the all-spaces flag.
struct NonBreakingText {
    text: String,
    width: f32,
    width_on_new_line: f32,
    is_whitespace: bool,
}

impl NonBreakingText {
    fn new(font: Option<&UiFont>, text: String) -> Self {
        let mut width = 0.0;
        let mut delta = 0.0;
        let mut is_whitespace = true;
        let spacing = char_spacing(font);
        for (i, c) in text.chars().enumerate() {
            if i == 0 {
                // `:33` — KernedWidthOnNewLine − KernedWidth.
                delta = kerned_width_new_line(font, c) - kerned_width(font, c);
            } else {
                width += spacing;
            }
            width += kerned_width(font, c);
            if c != ' ' {
                is_whitespace = false;
            }
        }
        Self {
            text,
            width,
            width_on_new_line: width + delta,
            is_whitespace,
        }
    }
}

/// The builder state (`WrappedTextBuilder.cs:50-71`): the completed text,
/// the working line and its measured width. `firstLineOffset` rides the
/// ctor (`:64-71`); the port's callers use 0.
struct WrappedTextBuilder<'a> {
    font: Option<&'a UiFont>,
    culture: GameCulture,
    max_width: f32,
    completed: String,
    working: Vec<char>,
    working_width: f32,
    first_line: bool,
}

impl<'a> WrappedTextBuilder<'a> {
    fn new(font: Option<&'a UiFont>, culture: GameCulture, max_width: f32) -> Self {
        Self {
            font,
            culture,
            max_width,
            completed: String::new(),
            working: Vec::new(),
            working_width: 0.0,
            first_line: true,
        }
    }

    /// `CommitWorkingLine` (`:73-83`).
    fn commit_working_line(&mut self) {
        if !self.first_line {
            self.completed.push('\n');
        }
        self.working_width = 0.0;
        self.completed.extend(self.working.iter());
        self.working.clear();
        self.first_line = false;
    }

    /// `FinishPartialLine` (`:85-91`).
    fn finish_partial_line(&mut self) {
        if self.working_width > 0.0 {
            self.commit_working_line();
        }
    }

    /// `Append` (`:93-115`): fit the token on the working line (spacing
    /// before it when the line is non-empty), else commit the partial line
    /// and start fresh — dropping a whitespace-only token, hard-breaking an
    /// over-wide word.
    fn append(&mut self, token: NonBreakingText) {
        let spacing = char_spacing(self.font);
        let num = if self.working_width != 0.0 {
            self.working_width + spacing + token.width
        } else {
            token.width_on_new_line
        };
        if num <= self.max_width {
            self.working.extend(token.text.chars());
            self.working_width = num;
            return;
        }
        self.finish_partial_line();
        if !token.is_whitespace {
            if token.width_on_new_line <= self.max_width {
                self.working.extend(token.text.chars());
                self.working_width = token.width_on_new_line;
            } else {
                self.append_with_hard_breaks(&token.text);
            }
        }
    }

    /// `AppendWithHardBreaks` (`:117-149`): char-by-char for a word wider
    /// than the whole line. Non-Chinese cultures hyphenate: push back
    /// trailing chars until the line + '-' fits (keeping >1 char), append
    /// '-', commit, and retry the pushed-back char (`i--` twice).
    fn append_with_hard_breaks(&mut self, text: &str) {
        let chars: Vec<char> = text.chars().collect();
        let spacing = char_spacing(self.font);
        let hyphen_w = kerned_width(self.font, '-');
        let mut i = 0usize;
        while i < chars.len() {
            let c = chars[i];
            let num = if !self.working.is_empty() {
                self.working_width + spacing + kerned_width(self.font, c)
            } else {
                kerned_width_new_line(self.font, c)
            };
            if num <= self.max_width {
                self.working.push(c);
                self.working_width = num;
            } else if self.working.len() > 1 && !breaks_between_most_glyphs(self.culture) {
                self.working_width += spacing + hyphen_w;
                while self.working.len() > 1 && self.working_width > self.max_width {
                    let last = *self.working.last().unwrap();
                    self.working_width -= spacing + kerned_width(self.font, last);
                    self.working.pop();
                    i -= 1;
                }
                self.working.push('-');
                self.finish_partial_line();
                i -= 1;
            } else {
                self.finish_partial_line();
                self.working.push(c);
                self.working_width = kerned_width_new_line(self.font, c);
            }
            i += 1;
        }
    }

    /// `Build` (`:151-170`): '\n' commits the working line, everything else
    /// flows through `ReadUntilBreakable` tokens.
    fn build(mut self, text: &str) -> String {
        let chars: Vec<char> = text.chars().collect();
        let mut pos = 0usize;
        // `stringReader.Peek() > 0` (`:155`): NUL ends the build too.
        while pos < chars.len() && chars[pos] != '\0' {
            if chars[pos] == '\n' {
                pos += 1;
                self.commit_working_line();
            } else {
                let token = read_until_breakable(&chars, &mut pos, self.culture);
                self.append(NonBreakingText::new(self.font, token));
            }
        }
        self.commit_working_line();
        self.completed
    }
}

/// `DynamicSpriteFont.CreateWrappedText(text, maxWidth, culture)`
/// (`DynamicSpriteFont.cs:329-341`): `maxWidth <= 0` returns the text
/// unchanged; otherwise the builder's wrapped string ('\n'-separated).
pub(crate) fn create_wrapped_text(
    font: Option<&UiFont>,
    text: &str,
    max_width: f32,
    culture: GameCulture,
) -> String {
    if max_width <= 0.0 {
        return text.to_string();
    }
    WrappedTextBuilder::new(font, culture, max_width).build(text)
}

/// `Utils.WordwrapString(text, font, maxWidth, maxLines)` (`Utils.cs:860-867`):
/// `CreateWrappedText` split on '\n', the first `min(len, maxLines)` rows
/// copied into a `maxLines`-slot array — the tail slots stay null in the C#;
/// the port returns only the filled rows.
pub(crate) fn wordwrap_string(
    font: Option<&UiFont>,
    text: &str,
    max_width: f32,
    max_lines: usize,
    culture: GameCulture,
) -> Vec<String> {
    let wrapped = create_wrapped_text(font, text, max_width, culture);
    wrapped
        .split('\n')
        .take(max_lines)
        .map(str::to_string)
        .collect()
}

/// `DynamicSpriteFont.MeasureString(text)` (`DynamicSpriteFont.cs:272-318`):
/// the full multi-line box — max line width, `LineSpacing` per row, the
/// per-line glyph `Padding.Height` floor (`:311`). The port's [`FontGlyph`]
/// keeps only the pad ORIGIN (`assets.rs:1519-1520`), so the height floor
/// falls back to the glyph's drawn height `h` — a recorded approximation
/// (docs/ANALYSIS.md "Overhead chat + text wrap (W4-d121)"). Empty text
/// measures (0, 0) (`:274-277`).
pub(crate) fn measure_string(font: Option<&UiFont>, text: &str) -> (f32, f32) {
    if text.is_empty() {
        return (0.0, 0.0);
    }
    let Some(f) = font else {
        // The bitmap fallback: 8 px/char, 8 px rows (the `panel_lines`
        // stand-in). Newlines split rows like the retail measure.
        let mut max_w = 0.0f32;
        let mut lines = 1usize;
        let mut cur = 0.0f32;
        for c in text.chars() {
            match c {
                '\n' => {
                    max_w = max_w.max(cur);
                    cur = 0.0;
                    lines += 1;
                }
                '\r' => {}
                _ => cur += 8.0,
            }
        }
        return (max_w.max(cur), lines as f32 * 8.0);
    };
    measure_retail(f, text)
}

/// The retail-sheet walk of [`measure_string`] — the `:278-317` loop.
fn measure_retail(f: &UiFont, text: &str) -> (f32, f32) {
    let mut x = 0.0f32;
    let mut y = f.line_spacing;
    let mut max_w = 0.0f32;
    let mut newlines = 0usize;
    let mut z = 0.0f32;
    let mut first = true;
    for c in text.chars() {
        match c {
            '\n' => {
                max_w = max_w.max(x + z.max(0.0));
                z = 0.0;
                x = 0.0;
                y = f.line_spacing;
                first = true;
                newlines += 1;
                continue;
            }
            '\r' => continue,
            _ => {}
        }
        let Some(g) = f.glyph(c) else { continue };
        if first {
            x += g.kerning[0].max(0.0);
        } else {
            x += f.spacing + z;
        }
        x += g.kerning[0] + g.kerning[1];
        z = g.kerning[2];
        // `:311` — `zero.Y = max(zero.Y, Padding.Height)`; the port's
        // `FontGlyph` keeps only the pad ORIGIN (`assets.rs:1519`), so the
        // drawn height `h` stands in (ANALYSIS "Overhead chat + text wrap").
        y = y.max(g.h);
        first = false;
    }
    x += z.max(0.0);
    y += newlines as f32 * f.line_spacing;
    (x.max(max_w), y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assets::FontGlyph;

    /// A synthetic font: every ASCII glyph 8 px wide (kerning (0, 8, 0)),
    /// spacing 0, line spacing 16 — the metrics arithmetic stays exact.
    fn flat_font() -> UiFont {
        let mut f = UiFont {
            line_spacing: 16.0,
            spacing: 0.0,
            fallback: '?',
            glyphs: Default::default(),
        };
        for c in ' '..='~' {
            f.glyphs.insert(
                c,
                FontGlyph {
                    uv: [0.0; 4],
                    w: 8.0,
                    h: 12.0,
                    pad: [0.0, 0.0],
                    kerning: [0.0, 8.0, 0.0],
                    layer: 1,
                },
            );
        }
        f
    }

    /// `WrappedTextBuilder.Append` (`:93-115`): greedy word fit — "aaaa
    /// bbbb" at 40 px (5 chars) commits "aaaa " (the space token fitted,
    /// `:96-100`) then wraps "bbbb".
    #[test]
    fn words_wrap_at_the_measured_width() {
        let f = flat_font();
        assert_eq!(
            create_wrapped_text(Some(&f), "aaaa bbbb", 40.0, GameCulture::English),
            "aaaa \nbbbb"
        );
        // Exactly-fitting lines stay whole (`num <= maxWidth`, `:96`).
        assert_eq!(
            create_wrapped_text(Some(&f), "aa bb", 40.0, GameCulture::English),
            "aa bb"
        );
    }

    /// `AppendWithHardBreaks` (`:117-149`): an over-wide word hyphenates —
    /// "aaaaaaaaaa" (10 chars) at 40 px keeps 4 chars + '-' (5×8 = 40).
    #[test]
    fn long_words_hard_break_with_a_hyphen() {
        let f = flat_font();
        assert_eq!(
            create_wrapped_text(Some(&f), "aaaaaaaaaa", 40.0, GameCulture::English),
            "aaaa-\naaaa-\naa"
        );
    }

    /// `Build` (`:157-161`): explicit newlines commit rows; trailing
    /// newlines keep the empty tail row (`CommitWorkingLine` runs once
    /// more at `:168`).
    #[test]
    fn explicit_newlines_commit_rows() {
        let f = flat_font();
        assert_eq!(
            create_wrapped_text(Some(&f), "one\ntwo", 1000.0, GameCulture::English),
            "one\ntwo"
        );
        assert_eq!(
            create_wrapped_text(Some(&f), "a\n", 1000.0, GameCulture::English),
            "a\n"
        );
    }

    /// `Utils.WordwrapString` (`Utils.cs:860-867`): the maxLines cap.
    #[test]
    fn wordwrap_caps_the_line_count() {
        let f = flat_font();
        let many = "line\n".repeat(30);
        assert_eq!(
            wordwrap_string(Some(&f), &many, 460.0, 10, GameCulture::English).len(),
            10
        );
    }

    /// The Chinese branch (`CanBreakBetween`, `:64-75`): CJK text breaks
    /// between glyphs, digits stay glued.
    #[test]
    fn chinese_breaks_between_glyphs_but_not_digits() {
        let mut f = flat_font();
        // The CJK glyphs join the flat 8 px table (the retail sheet carries
        // them; the synthetic stand-in keeps the arithmetic exact).
        for c in "你好世界啊".chars() {
            f.glyphs.insert(
                c,
                FontGlyph {
                    uv: [0.0; 4],
                    w: 8.0,
                    h: 12.0,
                    pad: [0.0, 0.0],
                    kerning: [0.0, 8.0, 0.0],
                    layer: 1,
                },
            );
        }
        // 4 chars fit at 32 px; the 5th wraps.
        assert_eq!(
            create_wrapped_text(Some(&f), "你好世界啊", 32.0, GameCulture::Chinese),
            "你好世界\n啊"
        );
        // The digit run "12345" never splits (Numeric, `:66-69`) — it
        // hard-breaks WITHOUT a hyphen in the Chinese branch (`:129`).
        assert_eq!(
            create_wrapped_text(Some(&f), "12345", 32.0, GameCulture::Chinese),
            "1234\n5"
        );
    }

    /// `MeasureString` (`:272-318`): two rows of "ab" measure (16, 32)
    /// with the flat font; empty text is (0, 0).
    #[test]
    fn measure_walks_rows_and_widths() {
        let f = flat_font();
        assert_eq!(measure_string(Some(&f), "ab\nab"), (16.0, 32.0));
        assert_eq!(measure_string(Some(&f), ""), (0.0, 0.0));
        // The bitmap fallback: 8 px/char, 8 px rows.
        assert_eq!(measure_string(None, "ab\nab"), (16.0, 16.0));
    }

    /// `CreateWrappedText`'s `maxWidth <= 0` guard (`:336-339`).
    #[test]
    fn nonpositive_width_returns_the_text() {
        assert_eq!(
            create_wrapped_text(None, "unchanged text", 0.0, GameCulture::English),
            "unchanged text"
        );
    }
}
