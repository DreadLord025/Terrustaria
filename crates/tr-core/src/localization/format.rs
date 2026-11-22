//! The `Lang.GetGlobalSubstitution` values, `VariableText` and the C#
//! `string.Format` clone used by the localized values.

/// One value of `Lang.GetGlobalSubstitution`.
#[derive(Clone, Debug, PartialEq)]
pub enum Substitution {
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(String),
}

impl Substitution {
    fn as_bool(&self) -> Option<bool> {
        match self {
            Substitution::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn display(&self) -> String {
        match self {
            Substitution::Bool(b) => if *b { "True" } else { "False" }.to_string(),
            Substitution::Int(i) => i.to_string(),
            Substitution::Float(f) => f.to_string(),
            Substitution::Text(t) => t.clone(),
        }
    }
}

impl From<&str> for Substitution {
    fn from(v: &str) -> Self {
        Substitution::Text(v.to_string())
    }
}

impl From<String> for Substitution {
    fn from(v: String) -> Self {
        Substitution::Text(v)
    }
}

impl From<bool> for Substitution {
    fn from(v: bool) -> Self {
        Substitution::Bool(v)
    }
}

impl From<i64> for Substitution {
    fn from(v: i64) -> Self {
        Substitution::Int(v)
    }
}

impl From<f64> for Substitution {
    fn from(v: f64) -> Self {
        Substitution::Float(v)
    }
}

/// `VariableText`: a value that contains `{Name}` / `{?Name}` / `{!Name}` tokens.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct VariableText {
    /// `<original>` (`ToString()`).
    original: String,
    /// `<format>` with variables rewritten to `{index}`.
    format: String,
    conditions: Vec<Condition>,
    variables: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct Condition {
    required_value: bool,
    name: String,
}

impl VariableText {
    /// `VariableText.TryCreate`, with the same `{(?!?)?([a-zA-Z][\w\.]*)}` shape.
    pub(super) fn try_create(s: &str) -> Option<VariableText> {
        let chars: Vec<char> = s.chars().collect();
        let mut format = String::new();
        let mut variables: Vec<String> = Vec::new();
        let mut conditions: Vec<Condition> = Vec::new();
        let mut found = false;

        let mut i = 0;
        while i < chars.len() {
            if chars[i] == '{' {
                if let Some((marker, name, next)) = parse_substitution(&chars, i) {
                    found = true;
                    if !marker.is_empty() {
                        conditions.push(Condition {
                            required_value: marker == "?",
                            name,
                        });
                    } else {
                        let index = match variables.iter().position(|v| *v == name) {
                            Some(index) => index,
                            None => {
                                variables.push(name);
                                variables.len() - 1
                            }
                        };
                        format.push('{');
                        format.push_str(&index.to_string());
                        format.push('}');
                    }
                    i = next;
                    continue;
                }
            }
            format.push(chars[i]);
            i += 1;
        }

        found.then(|| VariableText {
            original: s.to_string(),
            format,
            conditions,
            variables,
        })
    }

    /// `VariableText.TryFormat`: `None` when a condition or a variable is missing.
    pub(super) fn try_format(
        &self,
        lookup: &dyn Fn(&str) -> Option<Substitution>,
    ) -> Option<String> {
        for c in &self.conditions {
            let got = lookup(&c.name).and_then(|v| v.as_bool()).unwrap_or(false);
            if got != c.required_value {
                return None;
            }
        }
        let mut args = Vec::with_capacity(self.variables.len());
        for v in &self.variables {
            args.push(lookup(v)?.display());
        }
        Some(format_net(&self.format, &args))
    }
}

/// Matches `{(?!?)?([a-zA-Z][\w\.]*)}` at `chars[start] == '{'`. Returns the marker
/// (`""`, `"?"`, `"!"` or `"?!"`), the variable name and the index just past `}`.
fn parse_substitution(chars: &[char], start: usize) -> Option<(&'static str, String, usize)> {
    let mut j = start + 1;
    let mut marker = "";
    if chars.get(j) == Some(&'?') {
        j += 1;
        marker = "?";
        if chars.get(j) == Some(&'!') {
            j += 1;
            marker = "?!";
        }
    } else if chars.get(j) == Some(&'!') {
        j += 1;
        marker = "!";
    }
    let name_start = j;
    while let Some(c) = chars.get(j) {
        if c.is_ascii_alphanumeric() || *c == '_' || *c == '.' {
            j += 1;
        } else {
            break;
        }
    }
    let name: String = chars[name_start..j].iter().collect();
    if name.is_empty() || !chars[name_start].is_ascii_alphabetic() {
        return None;
    }
    if chars.get(j) != Some(&'}') {
        return None;
    }
    Some((marker, name, j + 1))
}

/// C# `string.Format` with `String` arguments: `{0}`, `{0:spec}` (the specifier is
/// honoured for the numeric section formats Terraria's language rows use —
/// `{0:+0;-#}`, `CommonItemTooltip.PrefixArmorPenetration`/`PrefixTagDamage` —
/// and ignored otherwise), and `{{` / `}}` escapes.
///
/// An out-of-range index keeps the original token instead of throwing
/// `FormatException`, so a malformed language file cannot crash the renderer.
pub fn format_net(format: &str, args: &[String]) -> String {
    let chars: Vec<char> = format.chars().collect();
    let mut out = String::with_capacity(format.len());
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            '{' if chars.get(i + 1) == Some(&'{') => {
                out.push('{');
                i += 2;
            }
            '{' => {
                let mut j = i + 1;
                let mut digits = String::new();
                while let Some(c) = chars.get(j) {
                    if c.is_ascii_digit() {
                        digits.push(*c);
                        j += 1;
                    } else {
                        break;
                    }
                }
                if digits.is_empty() {
                    out.push('{');
                    i += 1;
                    continue;
                }
                // Collect an optional format specifier.
                let spec_start = if chars.get(j) == Some(&':') { j + 1 } else { j };
                while let Some(c) = chars.get(j) {
                    if *c == '}' {
                        break;
                    }
                    j += 1;
                }
                if chars.get(j) != Some(&'}') {
                    out.push('{');
                    i += 1;
                    continue;
                }
                let spec: String = chars[spec_start..j].iter().collect();
                match digits.parse::<usize>().ok().and_then(|n| args.get(n)) {
                    Some(a) => out.push_str(&apply_format_spec(a, &spec)),
                    None => out.extend(chars[i..=j].iter()),
                }
                i = j + 1;
            }
            '}' if chars.get(i + 1) == Some(&'}') => {
                out.push('}');
                i += 2;
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// The numeric custom-format subset .NET's `int.ToString(specifier)` runs for
/// the tooltip's `{0:+0;-#}` rows (`CommonItemTooltip.PrefixArmorPenetration`
/// / `PrefixTagDamage`, `Main.cs:21141`/`:21148`): up to three `;`-separated
/// sections (positive;negative;zero), each a literal run with `0` mandatory
/// digit holes and `#` optional-digit holes, filled right-to-left. With one
/// section the sign prints outside it; with a negative section the magnitude
/// prints WITHOUT the sign (the section's literals carry it). Non-numeric
/// arguments and non-numeric specs pass through unchanged (the old ignore
/// behaviour).
fn apply_format_spec(arg: &str, spec: &str) -> String {
    if spec.is_empty() {
        return arg.to_string();
    }
    let Ok(value) = arg.parse::<i64>() else {
        return arg.to_string();
    };
    // Custom numeric format vs standard specifier (.NET's rule): a
    // multi-section spec is always custom; a single-section spec is standard
    // when it is one alphabetic char optionally followed by digits (N0, D5,
    // F2 — keep the ignore behaviour), custom otherwise.
    let multi_section = spec.contains(';');
    if !multi_section {
        let mut chars = spec.chars();
        if let Some(first) = chars.next() {
            if first.is_ascii_alphabetic() && chars.all(|c| c.is_ascii_digit()) {
                return arg.to_string();
            }
        }
    }
    if !spec.contains(['0', '#']) && !multi_section {
        return arg.to_string();
    }
    let sections: Vec<&str> = spec.split(';').collect();
    let (section, magnitude, sign) = if sections.len() >= 2 {
        if value > 0 {
            (sections[0], value as u64, "")
        } else if value < 0 {
            (sections[1], value.unsigned_abs(), "")
        } else {
            // .NET: an empty third section prints nothing; a missing one
            // reuses the positive section.
            (sections.get(2).copied().unwrap_or(sections[0]), 0u64, "")
        }
    } else if value < 0 {
        (sections[0], value.unsigned_abs(), "-")
    } else {
        (sections[0], value as u64, "")
    };
    let sec: Vec<char> = section.chars().collect();
    let holes: Vec<usize> = sec
        .iter()
        .enumerate()
        .filter(|(_, c)| matches!(c, '0' | '#'))
        .map(|(i, _)| i)
        .collect();
    if holes.is_empty() {
        return format!("{sign}{section}");
    }
    let digits = magnitude.to_string();
    // Fill the holes right-to-left; a '0' hole past the digit run's left edge
    // pads a zero, a '#' hole stays empty. Digits LEFT of the leftmost hole
    // prepend before it (.NET's custom-format overflow rule: "0" on 123 is
    // "123", not "3").
    let mut fill: Vec<Option<char>> = vec![None; holes.len()];
    let mut di = digits.len();
    for k in (0..holes.len()).rev() {
        if di > 0 {
            di -= 1;
            fill[k] = Some(digits.as_bytes()[di] as char);
        } else if sec[holes[k]] == '0' {
            fill[k] = Some('0');
        }
    }
    let overflow = &digits[..di];
    let mut rendered = String::new();
    let mut k = 0usize;
    let mut overflow_done = false;
    for (i, c) in sec.iter().enumerate() {
        match c {
            '0' | '#' => {
                if !overflow_done && i == holes[0] {
                    rendered.push_str(overflow);
                    overflow_done = true;
                }
                if let Some(d) = fill[k] {
                    rendered.push(d);
                }
                k += 1;
            }
            literal => rendered.push(*literal),
        }
    }
    format!("{sign}{rendered}")
}

#[cfg(test)]
mod w4_f28_tests {
    use super::format_net;

    /// W4-f28 — the `{0:+0;-#}` section format the prefix tooltip's armor
    /// penetration / tag damage rows use (`CommonItemTooltip
    /// .PrefixArmorPenetration`, `Main.cs:21141`/`:21148`): .NET's
    /// `(+5).ToString("+0;-#")` = "+5", `(-3)` = "-3", `(0)` = "+0"
    /// (the positive section renders zero when no third section exists).
    #[test]
    fn the_prefix_rows_section_format_matches_dotnet() {
        let f = "{0:+0;-#} armor penetration";
        assert_eq!(format_net(f, &["5".into()]), "+5 armor penetration");
        assert_eq!(format_net(f, &["-3".into()]), "-3 armor penetration");
        assert_eq!(format_net(f, &["0".into()]), "+0 armor penetration");
        assert_eq!(format_net(f, &["12".into()]), "+12 armor penetration");
        assert_eq!(format_net(f, &["-12".into()]), "-12 armor penetration");
    }

    /// The plain holes keep their behaviour — the spec-less rows and the
    /// two-argument `RestoresLifeRange` (`Main.cs:20881`).
    #[test]
    fn plain_and_two_arg_holes_still_format() {
        assert_eq!(
            format_net("Restores from {0} to {1} life", &["50".into(), "120".into()]),
            "Restores from 50 to 120 life"
        );
        assert_eq!(format_net("{{0}}", &[]), "{0}");
        // Out-of-range keeps the token (the no-FormatException rule).
        assert_eq!(format_net("{1}", &["a".into()]), "{1}");
        // A non-numeric argument passes through even with a numeric spec.
        assert_eq!(format_net("{0:+0;-#}", &["x".into()]), "x");
        // A standard (non-section) specifier keeps the ignore behaviour.
        assert_eq!(format_net("{0:N0}", &["1234".into()]), "1234");
    }

    /// The zero-padding and optional-digit holes of the custom grammar
    /// (`000` pads, `##` drops) — exercised so the fill loop cannot drift.
    #[test]
    fn zero_pad_and_optional_holes() {
        assert_eq!(format_net("{0:000}", &["7".into()]), "007");
        assert_eq!(format_net("{0:000}", &["-7".into()]), "-007");
        assert_eq!(format_net("{0:##}", &["7".into()]), "7");
        assert_eq!(format_net("{0:+0;-#;zero}", &["0".into()]), "zero");
        // Digits left of the leftmost hole prepend (.NET overflow rule).
        assert_eq!(format_net("{0:0}", &["123".into()]), "123");
        assert_eq!(format_net("{0:+0;-#}", &["-12".into()]), "-12");
    }
}
