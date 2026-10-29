//! `Terraria.Localization` port: `GameCulture`, `LocalizedText`, `VariableText` and
//! `LanguageManager`.
//!
//! The original ships its strings as embedded resources named
//! `Terraria.Localization.Content.<culture>.*.json`; the loader here takes the same
//! text and mirrors `LanguageManager.LoadLanguageFromFileTextJson`
//! (`Dictionary<string, Dictionary<string, string>>`), the `$` variant keys, the
//! `{$(Category.Key)}` copy commands and the `{Name}` variable substitution of
//! `VariableText`. Only the JSON path is ported — the shipped content is JSON and the
//! `.csv` path (`LoadLanguageFromFileTextCsv`) exists for mods only.
//!
//! `Lang.GetGlobalSubstitution` is modelled as a [`Substitution`] table on the
//! manager; `Lang`'s static methods are the corresponding `LanguageManager` methods.

use std::collections::HashMap;

use anyhow::{Context, Result};

/// `LanguageManager.VariationSeparatorSign`.
pub const VARIATION_SEPARATOR_SIGN: char = '$';

/// `GameCulture.CultureName` plus the culture names and legacy ids of `GameCulture`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum GameCulture {
    English,
    German,
    Italian,
    French,
    Spanish,
    Russian,
    Chinese,
    Portuguese,
    Polish,
    Japanese,
    Korean,
    ChineseTraditional,
}

/// The twelve shipped cultures, in the order `GameCulture` registers them.
pub const CULTURES: [GameCulture; 12] = [
    GameCulture::English,
    GameCulture::German,
    GameCulture::Italian,
    GameCulture::French,
    GameCulture::Spanish,
    GameCulture::Russian,
    GameCulture::Chinese,
    GameCulture::Portuguese,
    GameCulture::Polish,
    GameCulture::Japanese,
    GameCulture::Korean,
    GameCulture::ChineseTraditional,
];

impl GameCulture {
    /// `GameCulture.DefaultCulture`: English.
    pub const DEFAULT: GameCulture = GameCulture::English;

    /// `CultureInfo.Name`.
    pub fn name(self) -> &'static str {
        match self {
            GameCulture::English => "en-US",
            GameCulture::German => "de-DE",
            GameCulture::Italian => "it-IT",
            GameCulture::French => "fr-FR",
            GameCulture::Spanish => "es-ES",
            GameCulture::Russian => "ru-RU",
            GameCulture::Chinese => "zh-Hans",
            GameCulture::Portuguese => "pt-BR",
            GameCulture::Polish => "pl-PL",
            GameCulture::Japanese => "ja-JP",
            GameCulture::Korean => "ko-KR",
            GameCulture::ChineseTraditional => "zh-Hant",
        }
    }

    /// `GameCulture.LegacyId`.
    pub fn legacy_id(self) -> i32 {
        match self {
            GameCulture::English => 1,
            GameCulture::German => 2,
            GameCulture::Italian => 3,
            GameCulture::French => 4,
            GameCulture::Spanish => 5,
            GameCulture::Russian => 6,
            GameCulture::Chinese => 7,
            GameCulture::Portuguese => 8,
            GameCulture::Polish => 9,
            GameCulture::Japanese => 10,
            GameCulture::Korean => 11,
            GameCulture::ChineseTraditional => 12,
        }
    }

    /// `GameCulture.FromLegacyId`: ids below 1 clamp to 1, unknown ids fall back to
    /// the default culture.
    pub fn from_legacy_id(id: i32) -> GameCulture {
        let id = id.max(1);
        CULTURES
            .iter()
            .copied()
            .find(|c| c.legacy_id() == id)
            .unwrap_or(GameCulture::DEFAULT)
    }

    /// `GameCulture.FromName`.
    pub fn from_name(name: &str) -> GameCulture {
        CULTURES
            .iter()
            .copied()
            .find(|c| c.name() == name)
            .unwrap_or(GameCulture::DEFAULT)
    }

    /// `GameCulture.FromCultureName`.
    pub fn from_culture_name(index: usize) -> GameCulture {
        CULTURES.get(index).copied().unwrap_or(GameCulture::DEFAULT)
    }

    /// `LanguageManager.GetLanguageFilesForCulture` filter: the manifest resource names
    /// that belong to this culture.
    pub fn matches_resource(name: &str, culture: GameCulture) -> bool {
        let prefix = format!("Terraria.Localization.Content.{}", culture.name());
        name.starts_with(&prefix) && name.ends_with(".json")
    }
}

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
struct VariableText {
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
    fn try_create(s: &str) -> Option<VariableText> {
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
    fn try_format(&self, lookup: &dyn Fn(&str) -> Option<Substitution>) -> Option<String> {
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

/// `LocalizedText`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LocalizedText {
    key: String,
    /// `_value.ToString()`: the raw loaded string.
    value: String,
    /// `EnglishValue`: the value while the default culture is active.
    english_value: String,
    parsed: Option<VariableText>,
}

impl LocalizedText {
    /// `new LocalizedText(key, text)`; `english_value` is only remembered while the
    /// default culture is active (`LocalizedText.SetValue`).
    pub fn new(key: impl Into<String>, text: impl Into<String>, english: bool) -> Self {
        let key = key.into();
        let text = text.into();
        let parsed = VariableText::try_create(&text);
        LocalizedText {
            english_value: if english { text.clone() } else { String::new() },
            key,
            value: text,
            parsed,
        }
    }

    /// `LocalizedText.Empty`.
    pub fn empty() -> Self {
        LocalizedText::new("", "", true)
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    /// `UnformattedValue`.
    pub fn unformatted_value(&self) -> &str {
        &self.value
    }

    /// `EnglishValue`.
    pub fn english_value(&self) -> &str {
        &self.english_value
    }

    /// `HasValue`.
    pub fn has_value(&self) -> bool {
        self.english_value != self.key
    }

    /// `SetValue`, called after a load: only the default culture records the English
    /// value.
    fn set_value(&mut self, value: impl Into<String>, english: bool) {
        let value = value.into();
        if english {
            self.english_value = value.clone();
        }
        self.parsed = VariableText::try_create(&value);
        self.value = value;
    }

    /// `SetValue(localizedText.Key)` from `SetAllTextValuesToKeys`.
    fn set_value_to_key(&mut self) {
        let key = self.key.clone();
        self.set_value(key, false);
    }

    /// `LocalizedText.Value`: the raw value with `{Name}` substitutions applied, or the
    /// original text when a condition/variable is missing.
    pub fn value_with(&self, lookup: &dyn Fn(&str) -> Option<Substitution>) -> String {
        if let Some(parsed) = &self.parsed {
            if let Some(text) = parsed.try_format(lookup) {
                return text;
            }
        }
        self.value.clone()
    }

    /// `LocalizedText.Format(params object[] args)`.
    pub fn format(&self, lookup: &dyn Fn(&str) -> Option<Substitution>, args: &[String]) -> String {
        format_net(&self.value_with(lookup), args)
    }
}

/// `LanguageManager`.
#[derive(Clone, Debug)]
pub struct LanguageManager {
    localized_texts: HashMap<String, LocalizedText>,
    category_grouped_keys: HashMap<String, Vec<String>>,
    text_variations: HashMap<String, HashMap<String, String>>,
    fallback_culture: GameCulture,
    /// `ActiveCulture` starts null in the original (the auto-property default), so the
    /// first `SetLanguage` always reloads even for the default culture.
    active_culture: Option<GameCulture>,
    substitutions: HashMap<String, Substitution>,
    /// `(manifest resource name, json text)` pairs, standing in for the embedded
    /// resources `Assembly.GetManifestResourceNames()` enumerates.
    sources: Vec<(String, String)>,
}

impl Default for LanguageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageManager {
    pub fn new() -> Self {
        let mut localized_texts = HashMap::new();
        localized_texts.insert(String::new(), LocalizedText::empty());
        LanguageManager {
            localized_texts,
            category_grouped_keys: HashMap::new(),
            text_variations: HashMap::new(),
            fallback_culture: GameCulture::DEFAULT,
            active_culture: None,
            substitutions: HashMap::new(),
            sources: Vec::new(),
        }
    }

    /// `LanguageManager.ActiveCulture`; the default culture until a load has happened.
    pub fn active_culture(&self) -> GameCulture {
        self.active_culture.unwrap_or(GameCulture::DEFAULT)
    }

    pub fn fallback_culture(&self) -> GameCulture {
        self.fallback_culture
    }

    /// `LanguageManager.SetFallbackCulture`.
    pub fn set_fallback_culture(&mut self, culture: GameCulture) {
        self.fallback_culture = culture;
    }

    /// Registers one embedded-resource stand-in (`GetLanguageFilesForCulture`).
    pub fn add_source(&mut self, resource_name: impl Into<String>, text: impl Into<String>) {
        self.sources.push((resource_name.into(), text.into()));
    }

    /// `Language`/`Lang.SetGlobalSubstitution`.
    pub fn set_substitution(&mut self, name: impl Into<String>, value: impl Into<Substitution>) {
        self.substitutions.insert(name.into(), value.into());
    }

    /// `Lang.RemoveGlobalSubstitution`.
    pub fn remove_substitution(&mut self, name: &str) {
        self.substitutions.remove(name);
    }

    /// `Lang.GetGlobalSubstitution` (`null` when unset).
    pub fn substitution(&self, name: &str) -> Option<Substitution> {
        self.substitutions.get(name).cloned()
    }

    /// `LanguageManager.SetLanguage`.
    pub fn set_language(&mut self, culture: GameCulture) {
        if self.active_culture != Some(culture) {
            self.reload_language(culture);
        }
    }

    /// `LanguageManager.ReloadLanguage`.
    fn reload_language(&mut self, target_culture: GameCulture) {
        if self.active_culture != Some(self.fallback_culture) {
            self.set_all_text_values_to_keys();
            if target_culture != self.fallback_culture {
                self.load_language(self.fallback_culture);
            }
        }
        self.load_language(target_culture);
        // `OnLanguageChanged` has no subscriber yet.
    }

    /// `LanguageManager.LoadLanguage`.
    fn load_language(&mut self, culture: GameCulture) {
        self.active_culture = Some(culture);
        self.text_variations.clear();
        self.load_files_for_culture(culture);
        self.process_copy_commands_in_texts();
    }

    /// `LanguageManager.LoadFilesForCulture`: the failed-file `catch`/`break` of the
    /// original is a silent skip here, so one bad file cannot abort the load.
    fn load_files_for_culture(&mut self, culture: GameCulture) {
        let files: Vec<(String, String)> = self
            .sources
            .iter()
            .filter(|(name, _)| GameCulture::matches_resource(name, culture))
            .cloned()
            .collect();
        for (name, text) in files {
            if text.len() < 2 {
                continue;
            }
            if let Err(e) = self.load_language_from_file_text_json(&text, true) {
                eprintln!("Failed to load language file: {name}: {e}");
            }
        }
    }

    /// `LanguageManager.SetAllTextValuesToKeys`.
    fn set_all_text_values_to_keys(&mut self) {
        for text in self.localized_texts.values_mut() {
            text.set_value_to_key();
        }
    }

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

    /// `LanguageManager.UpdateTextValue`.
    pub fn update_text_value(&mut self, key: &str, value: &str) -> bool {
        if key.contains(VARIATION_SEPARATOR_SIGN) {
            let parts: Vec<&str> = key.split(VARIATION_SEPARATOR_SIGN).collect();
            if parts.len() >= 2 {
                self.add_variant(parts[0], parts[1], value);
            }
            return true;
        }
        if let Some(text) = self.localized_texts.get_mut(key) {
            text.set_value(value, self.active_culture == Some(GameCulture::DEFAULT));
            return true;
        }
        false
    }

    /// `LanguageManager.AddVariant`.
    fn add_variant(&mut self, key: &str, variant: &str, value: &str) {
        self.text_variations
            .entry(key.to_string())
            .or_default()
            .insert(variant.to_string(), value.to_string());
    }

    /// `LanguageManager.TryGetVariation`.
    pub fn try_get_variation(&self, key: &str, variant: &str) -> Option<&str> {
        self.text_variations
            .get(key)
            .and_then(|v| v.get(variant))
            .map(String::as_str)
    }

    /// `LanguageManager.ProcessCopyCommandsInTexts`: expands `{$(Category.Key)}` until
    /// the text stops changing (the original caps this at 100 iterations).
    fn process_copy_commands_in_texts(&mut self) {
        let keys: Vec<String> = self.localized_texts.keys().cloned().collect();
        for key in keys {
            for _ in 0..100 {
                let current = self
                    .localized_texts
                    .get(&key)
                    .map(|t| t.unformatted_value().to_string())
                    .unwrap_or_default();
                let next = expand_copy_commands(&current, &self.localized_texts);
                if next == current {
                    break;
                }
                if let Some(text) = self.localized_texts.get_mut(&key) {
                    text.set_value(next, false);
                }
            }
        }
    }

    /// `LanguageManager.GetCategorySize`.
    pub fn category_size(&self, name: &str) -> usize {
        self.category_grouped_keys.get(name).map_or(0, Vec::len)
    }

    /// `LanguageManager.Exists`.
    pub fn exists(&self, key: &str) -> bool {
        self.localized_texts.contains_key(key)
    }

    /// `LanguageManager.GetText`, inserting the `key → key` placeholder of the
    /// original for unknown keys.
    pub fn get_text(&mut self, key: &str) -> &LocalizedText {
        if !self.localized_texts.contains_key(key) {
            let text = LocalizedText::new(key, key, false);
            self.localized_texts.insert(key.to_string(), text);
        }
        &self.localized_texts[key]
    }

    /// `LanguageManager.GetTextValue(key)`.
    pub fn get_text_value(&self, key: &str) -> String {
        match self.localized_texts.get(key) {
            Some(text) => text.value_with(&|name| self.substitution(name)),
            None => key.to_string(),
        }
    }

    /// `LanguageManager.GetTextValue(key, params object[] args)`.
    pub fn get_text_value_args(&self, key: &str, args: &[String]) -> String {
        match self.localized_texts.get(key) {
            Some(text) => text.format(&|name| self.substitution(name), args),
            None => key.to_string(),
        }
    }

    /// `LanguageManager.RandomFromCategory`, taking the RNG index explicitly so the
    /// caller controls determinism.
    pub fn random_from_category(&mut self, category: &str, index: usize) -> LocalizedText {
        let Some(keys) = self.category_grouped_keys.get(category) else {
            return LocalizedText::new(
                format!("{category}.RANDOM"),
                format!("{category}.RANDOM"),
                false,
            );
        };
        if keys.is_empty() {
            return LocalizedText::new(
                format!("{category}.RANDOM"),
                format!("{category}.RANDOM"),
                false,
            );
        }
        let name = keys[index % keys.len()].clone();
        self.get_text(&format!("{category}.{name}")).clone()
    }

    /// `LanguageManager.IndexedFromCategory`.
    pub fn indexed_from_category(&mut self, category: &str, index: usize) -> LocalizedText {
        let Some(keys) = self.category_grouped_keys.get(category) else {
            return LocalizedText::new(
                format!("{category}.INDEXED"),
                format!("{category}.INDEXED"),
                false,
            );
        };
        if keys.is_empty() {
            return LocalizedText::new(
                format!("{category}.INDEXED"),
                format!("{category}.INDEXED"),
                false,
            );
        }
        let name = keys[index % keys.len()].clone();
        self.get_text(&format!("{category}.{name}")).clone()
    }

    /// `LanguageManager.FindAll(LanguageSearchFilter)`.
    pub fn find_all(&self, filter: impl Fn(&str, &LocalizedText) -> bool) -> Vec<LocalizedText> {
        self.localized_texts
            .iter()
            .filter(|(k, v)| filter(k, v))
            .map(|(_, v)| v.clone())
            .collect()
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
fn expand_copy_commands(text: &str, texts: &HashMap<String, LocalizedText>) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn manager_with(json: &str) -> LanguageManager {
        let mut m = LanguageManager::new();
        m.add_source("Terraria.Localization.Content.en-US.json", json);
        m.set_language(GameCulture::English);
        m
    }

    #[test]
    fn cultures_match_the_original_ids_and_names() {
        assert_eq!(GameCulture::DEFAULT, GameCulture::English);
        assert_eq!(GameCulture::English.legacy_id(), 1);
        assert_eq!(GameCulture::ChineseTraditional.legacy_id(), 12);
        assert_eq!(GameCulture::from_name("ru-RU"), GameCulture::Russian);
        assert_eq!(GameCulture::from_legacy_id(7), GameCulture::Chinese);
        assert_eq!(GameCulture::from_legacy_id(0), GameCulture::English);
        assert_eq!(GameCulture::from_legacy_id(9999), GameCulture::English);
        assert_eq!(GameCulture::from_name("xx-XX"), GameCulture::English);
        assert!(GameCulture::matches_resource(
            "Terraria.Localization.Content.en-US.Items.json",
            GameCulture::English
        ));
        assert!(!GameCulture::matches_resource(
            "Terraria.Localization.Content.en-US.Items.json",
            GameCulture::German
        ));
    }

    #[test]
    fn json_creates_categories_and_english_values() {
        let json = r#"{
            "ItemName": { "DirtBlock": "Dirt Block", "StoneBlock": "Stone Block" },
            "ItemTooltip": { "Tooltip0": "-Item-" }
        }"#;
        let mut m = manager_with(json);
        assert_eq!(m.get_text_value("ItemName.DirtBlock"), "Dirt Block");
        assert_eq!(m.category_size("ItemName"), 2);
        assert_eq!(m.category_size("ItemTooltip"), 1);
        assert!(m.get_text("ItemName.DirtBlock").has_value());
        assert_eq!(
            m.get_text("ItemName.DirtBlock").english_value(),
            "Dirt Block"
        );
        // Unknown keys keep the key itself (`GetTextValue`).
        assert_eq!(m.get_text_value("Missing.Key"), "Missing.Key");
    }

    #[test]
    fn later_files_override_earlier_values() {
        let mut m = LanguageManager::new();
        m.add_source(
            "Terraria.Localization.Content.en-US.json",
            r#"{"A":{"K":"first"}}"#,
        );
        m.add_source(
            "Terraria.Localization.Content.en-US.Game.json",
            r#"{"A":{"K":"second","L":"new"}}"#,
        );
        m.set_language(GameCulture::English);
        assert_eq!(m.get_text_value("A.K"), "second");
        assert_eq!(m.get_text_value("A.L"), "new");
        assert_eq!(m.category_size("A"), 2);
    }

    #[test]
    fn dollar_keys_become_variations() {
        let json = r#"{
            "Town": {
                "Chatter$Day": "Nice day.",
                "Chatter$Night": "It is dark."
            }
        }"#;
        let m = manager_with(json);
        assert_eq!(
            m.try_get_variation("Town.Chatter", "Day"),
            Some("Nice day.")
        );
        assert_eq!(
            m.try_get_variation("Town.Chatter", "Night"),
            Some("It is dark.")
        );
        assert!(!m.exists("Town.Chatter$Day"));
    }

    #[test]
    fn copy_commands_expand_into_the_referenced_value() {
        let json = r#"{
            "Game": { "Name": "Terraria", "Title": "Welcome to {$Game.Name}!" },
            "Chain": { "B": "{$Game.Title}" }
        }"#;
        let m = manager_with(json);
        assert_eq!(m.get_text_value("Game.Title"), "Welcome to Terraria!");
        assert_eq!(m.get_text_value("Chain.B"), "Welcome to Terraria!");
    }

    #[test]
    fn format_substitutes_arguments_and_keeps_escapes() {
        let json = r#"{
            "Tooltip": {
                "UsesMana": "Uses {0} mana",
                "Range": "Restores from {0} to {1} life",
                "Braces": "{{literal}} {0}"
            }
        }"#;
        let m = manager_with(json);
        assert_eq!(
            m.get_text_value_args("Tooltip.UsesMana", &["20".into()]),
            "Uses 20 mana"
        );
        assert_eq!(
            m.get_text_value_args("Tooltip.Range", &["5".into(), "10".into()]),
            "Restores from 5 to 10 life"
        );
        assert_eq!(
            m.get_text_value_args("Tooltip.Braces", &["x".into()]),
            "{literal} x"
        );
        // A missing argument keeps the token instead of throwing.
        assert_eq!(
            m.get_text_value_args("Tooltip.UsesMana", &[]),
            "Uses {0} mana"
        );
    }

    #[test]
    fn variable_text_uses_substitutions_and_conditions() {
        let json = r#"{
            "Game": {
                "Hi": "Hello {PlayerName}",
                "NightOnly": "{?IsNight}The stars are out",
                "NotNight": "{!IsNight}The sun is out"
            }
        }"#;
        let mut m = manager_with(json);
        m.set_substitution("PlayerName", "Jeff");
        assert_eq!(m.get_text_value("Game.Hi"), "Hello Jeff");
        assert_eq!(
            m.get_text_value("Game.NightOnly"),
            "{?IsNight}The stars are out"
        );
        m.set_substitution("IsNight", true);
        assert_eq!(m.get_text_value("Game.NightOnly"), "The stars are out");
        assert_eq!(
            m.get_text_value("Game.NotNight"),
            "{!IsNight}The sun is out"
        );
        m.set_substitution("IsNight", false);
        assert_eq!(m.get_text_value("Game.NotNight"), "The sun is out");
    }

    #[test]
    fn unknown_key_becomes_its_own_placeholder_text() {
        let mut m = manager_with(r#"{"A":{"K":"v"}}"#);
        assert_eq!(m.get_text("Missing.Thing").key(), "Missing.Thing");
        assert_eq!(
            m.get_text("Missing.Thing").unformatted_value(),
            "Missing.Thing"
        );
    }

    #[test]
    fn switching_to_a_culture_reloads_fallback_then_target() {
        let mut m = LanguageManager::new();
        m.add_source(
            "Terraria.Localization.Content.en-US.Items.json",
            r#"{"ItemName":{"DirtBlock":"Dirt Block","StoneBlock":"Stone Block"}}"#,
        );
        m.add_source(
            "Terraria.Localization.Content.de-DE.Items.json",
            r#"{"ItemName":{"DirtBlock":"Erdblock"}}"#,
        );
        m.set_language(GameCulture::German);
        assert_eq!(m.get_text_value("ItemName.DirtBlock"), "Erdblock");
        // Missing in German, so the English fallback value survives.
        assert_eq!(m.get_text_value("ItemName.StoneBlock"), "Stone Block");
        // Switching back restores English even for the German override.
        m.set_language(GameCulture::English);
        assert_eq!(m.get_text_value("ItemName.DirtBlock"), "Dirt Block");
    }

    #[test]
    fn indexed_category_walks_file_order() {
        let json = r#"{"Dialogue":{"A":"first","B":"second","C":"third"}}"#;
        let mut m = manager_with(json);
        assert_eq!(
            m.indexed_from_category("Dialogue", 0).unformatted_value(),
            "first"
        );
        assert_eq!(
            m.indexed_from_category("Dialogue", 3).unformatted_value(),
            "first"
        );
        assert_eq!(
            m.indexed_from_category("Nope", 0).unformatted_value(),
            "Nope.INDEXED"
        );
    }

    /// Loads the strings the game ships in `Terraria.Localization.Content.en-US*.json`
    /// and checks a few known keys. Skipped when the decompiled resources are absent.
    #[test]
    fn shipped_english_resources_match_the_game() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../decompiled");
        let resources = [
            "Terraria.Localization.Content.en-US.json",
            "Terraria.Localization.Content.en-US.Items.json",
            "Terraria.Localization.Content.en-US.Game.json",
            "Terraria.Localization.Content.en-US.NPCs.json",
            "Terraria.Localization.Content.en-US.Projectiles.json",
            "Terraria.Localization.Content.en-US.Town.json",
            "Terraria.Localization.Content.en-US.Legacy.json",
        ];
        let mut manager = LanguageManager::new();
        let mut loaded = 0;
        for name in resources {
            let path = dir.join(name);
            if let Ok(text) = std::fs::read_to_string(&path) {
                manager.add_source(name, text);
                loaded += 1;
            }
        }
        if loaded == 0 {
            eprintln!("skipping: {} has no localization resources", dir.display());
            return;
        }
        manager.set_language(GameCulture::English);
        assert_eq!(manager.get_text_value("ItemName.DirtBlock"), "Dirt Block");
        assert_eq!(manager.get_text_value("ItemName.Shuriken"), "Shuriken");
        assert_eq!(
            manager.get_text_value_args("CommonItemTooltip.UsesMana", &["20".into()]),
            "Uses 20 mana"
        );
        assert!(
            manager.category_size("ItemName") > 1000,
            "ItemName category has {} keys",
            manager.category_size("ItemName")
        );
        // Every shipped file is a flat two-level map.
        assert!(manager.exists("Error.LoadFailed"));
    }
}
