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
            Substitution::Bool(b) => b.to_string(),
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
/// ignoreed, as Terraria only uses `{0}`-style holes), and `{{` / `}}` escapes.
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
                // Skip an optional format specifier.
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
                match digits.parse::<usize>().ok().and_then(|n| args.get(n)) {
                    Some(a) => out.push_str(a),
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
