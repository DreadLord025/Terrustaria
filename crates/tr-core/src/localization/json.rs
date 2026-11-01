//! The JSON resource-file loader: `LanguageManager.LoadLanguageFromFileTextJson`, the
//! trailing-comma shim for the Newtonsoft-flavoured JSON and the `{$(Category.Key)}`
//! copy commands.

use std::collections::HashMap;

use anyhow::{Context, Result};

use super::{GameCulture, LanguageManager, LocalizedText};

impl LanguageManager {
    /// `LanguageManager.LoadLanguageFromFileTextJson`. Returns an error where
    /// Newtonsoft would throw (captured by the caller).
    pub fn load_language_from_file_text_json(
        &mut self,
        file_text: &str,
        can_create_categories: bool,
    ) -> Result<()> {
        let parsed: serde_json::Value =
            serde_json::from_str(&strip_trailing_commas(file_text)).context("localization json")?;
        let categories = parsed.as_object().context("localization root object")?;
        for (category, entries) in categories {
            let entries = entries
                .as_object()
                .with_context(|| format!("localization category '{category}' is not an object"))?;
            for (name, value) in entries {
                let value = value
                    .as_str()
                    .with_context(|| format!("localization value {category}.{name}"))?;
                let key = format!("{category}.{name}");
                if !self.update_text_value(&key, value) && can_create_categories {
                    let english = self.active_culture == Some(GameCulture::DEFAULT);
                    self.localized_texts
                        .insert(key.clone(), LocalizedText::new(&key, value, english));
                    self.category_grouped_keys
                        .entry(category.clone())
                        .or_default()
                        .push(name.clone());
                }
            }
        }
        Ok(())
    }
}

/// The shipped resources are read by Newtonsoft, which accepts a trailing comma before a
/// closing brace/bracket; `serde_json` does not. Remove those commas (outside string
/// literals) so the game's own files parse unchanged.
fn strip_trailing_commas(json: &str) -> String {
    let chars: Vec<char> = json.chars().collect();
    let mut out = String::with_capacity(json.len());
    let mut in_string = false;
    let mut escaped = false;
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if in_string {
            out.push(c);
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_string = true;
            out.push(c);
            i += 1;
        } else if c == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                i += 1; // drop the trailing comma
            } else {
                out.push(c);
                i += 1;
            }
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// `LanguageManager`'s `{\$(\w+\.\w+)}` replacement, driven by the localized table.
pub(super) fn expand_copy_commands(text: &str, texts: &HashMap<String, LocalizedText>) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'$') {
            if let Some((key, next)) = parse_copy_command(&chars, i) {
                match texts.get(&key) {
                    Some(text) => out.push_str(text.unformatted_value()),
                    None => out.push_str(&key),
                }
                i = next;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// Matches `{\$(\w+\.\w+)}` at `chars[start]`.
fn parse_copy_command(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut j = start + 2; // skip `{` and `$`
    let mut key = String::new();
    let mut seen_dot = false;
    while let Some(c) = chars.get(j) {
        if c.is_ascii_alphanumeric() || *c == '_' {
            key.push(*c);
            j += 1;
        } else if *c == '.' && !seen_dot {
            seen_dot = true;
            key.push('.');
            j += 1;
        } else {
            break;
        }
    }
    // `\w+\.\w+` requires at least one word char on each side of the dot.
    if !seen_dot || key.ends_with('.') || key.starts_with('.') || chars.get(j) != Some(&'}') {
        return None;
    }
    Some((key, j + 1))
}
